//! Tests for rescaling the UI after the window changes.
//!
//! The bug these cover: `UiScale` is recomputed every frame, but a `Val::Px`
//! written into a `Node` at spawn time is not, so the layout kept whatever size
//! the window had when the UI was built. Only a hover corrected it, because
//! `buttons::apply_visual` happens to read the live scale - which is why the
//! buttons looked right at the pointer and wrong everywhere else.

use bevy::prelude::*;

use super::{ScaledFont, ScaledNode, UiScale, rescale_ui_system};

/// The inventory slot size, so the slot test states a real number from the game
/// rather than a made-up one.
const SLOT: f32 = 85.0;

fn app_at_scale(scale: f32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(UiScale(scale));
    app.add_systems(Update, rescale_ui_system);
    app
}

/// The single entity carrying `C`, read as a clone of its component.
fn read<T>(app: &mut App, entity: Entity) -> T
where
    T: Component + Clone,
{
    app.world().entity(entity).get::<T>().unwrap().clone()
}

/// Spawns one node with a design-space size, and returns its entity.
fn spawn_scaled(app: &mut App, design: ScaledNode, painted: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            design,
            Node {
                width: Val::Px(painted.x),
                height: Val::Px(painted.y),
                ..default()
            },
        ))
        .id()
}

fn only(app: &mut App) -> Entity {
    app.world_mut()
        .query_filtered::<Entity, With<ScaledNode>>()
        .single(app.world())
        .unwrap()
}

fn px(value: Val, context: &str) -> f32 {
    match value {
        Val::Px(px) => px,
        other => panic!("{context}: expected pixels, got {other:?}"),
    }
}

/// The button case, and the reason it is worth a test: without the rescale the
/// node keeps its spawn-time size, so a window twice as tall leaves the button at
/// half the size it should be.
#[test]
fn a_button_follows_the_window_height() {
    let mut app = app_at_scale(1.0);
    let button = spawn_scaled(&mut app, ScaledNode::sized(150.0, 50.0), Vec2::new(150.0, 50.0));
    app.update();

    let node = read::<Node>(&mut app, button);
    assert_eq!(px(node.width, "at scale 1"), 150.0, "the design size at scale 1");

    // The window doubles: 720 -> 1440 of design height.
    app.insert_resource(UiScale(2.0));
    app.update();
    let node = read::<Node>(&mut app, button);
    assert_eq!(px(node.width, "after doubling"), 300.0, "the button did not follow the window");
}

/// The same in the other direction, which is the one that made the buttons look
/// oversized: the window shrinks and the button has to shrink with it.
#[test]
fn a_button_shrinks_with_the_window() {
    let mut app = app_at_scale(1.5);
    let button = spawn_scaled(&mut app, ScaledNode::sized(150.0, 50.0), Vec2::new(225.0, 75.0));
    app.update();

    app.insert_resource(UiScale(0.7));
    app.update();
    let node = read::<Node>(&mut app, button);
    assert!(
        (px(node.width, "after shrinking") - 105.0).abs() < 0.01,
        "expected 105 at scale 0.7, got {}",
        px(node.width, "after shrinking"),
    );
}

/// The inventory case, which never worked at all: nothing in the inventory runs
/// `apply_visual`, so there was no hover to correct it either.
#[test]
fn an_inventory_slot_follows_the_window() {
    let mut app = app_at_scale(1.0);
    let slot = spawn_scaled(&mut app, ScaledNode::sized(SLOT, SLOT), Vec2::splat(SLOT));
    app.update();

    app.insert_resource(UiScale(2.0));
    app.update();
    let node = read::<Node>(&mut app, slot);
    assert_eq!(px(node.width, "slot"), SLOT * 2.0, "a slot did not follow the window");
}

/// Offsets and gaps, not just the size: the HUD sits at a margin, and a margin that
/// does not move leaves the interface hugging the window edge.
#[test]
fn offsets_and_gaps_follow_the_window() {
    let mut app = app_at_scale(1.0);
    app.world_mut().spawn((
        ScaledNode {
            bottom: Some(20.0),
            left: Some(12.0),
            gap: Some(8.0),
            padding: Some(18.0),
            border: Some(1.5),
            radius: Some(6.0),
            ..default()
        },
        Node::default(),
    ));
    app.update();
    let hud = only(&mut app);

    app.insert_resource(UiScale(2.0));
    app.update();
    let node = read::<Node>(&mut app, hud);

    assert_eq!(px(node.bottom, "bottom"), 40.0, "the bottom margin did not scale");
    assert_eq!(px(node.left, "left"), 24.0, "the left margin did not scale");
    assert_eq!(px(node.row_gap, "row gap"), 16.0, "the row gap did not scale");
    assert_eq!(px(node.column_gap, "column gap"), 16.0, "the column gap did not scale");
    assert_eq!(px(node.padding.bottom, "padding"), 36.0, "the padding did not scale");
    assert_eq!(px(node.border.top, "border"), 3.0, "the border did not scale");
    assert_eq!(
        px(node.border_radius.top_left, "radius"),
        12.0,
        "the corner radius did not scale",
    );
}

/// Font size lives in `TextFont` rather than in `Node`, so it needs its own
/// component and its own assertion - a scaled frame with tiny text is still broken.
#[test]
fn font_size_follows_the_window() {
    let mut app = app_at_scale(1.0);
    let label = app
        .world_mut()
        .spawn((
            ScaledFont(20.0),
            TextFont {
                font_size: FontSize::Px(20.0),
                ..default()
            },
        ))
        .id();
    app.update();

    app.insert_resource(UiScale(1.5));
    app.update();
    let font = read::<TextFont>(&mut app, label);
    assert_eq!(font.font_size, FontSize::Px(30.0));
}

/// Guarded on `is_changed`, so a window that is not moving costs nothing.
///
/// Proved by leaving a deliberately wrong value in place: a system that recomputed
/// unconditionally would overwrite it with the correct number and this would pass
/// for the wrong reason.
#[test]
fn nothing_is_touched_when_the_scale_is_unchanged() {
    let mut app = app_at_scale(2.0);
    let node_entity = spawn_scaled(&mut app, ScaledNode::sized(150.0, 50.0), Vec2::new(1.0, 1.0));

    // The first frame legitimately recomputes: inserting the resource counts as
    // changing it. So that is where the wrong value gets corrected...
    app.update();
    assert_eq!(
        px(read::<Node>(&mut app, node_entity).width, "first frame"),
        300.0,
        "the first frame should apply the scale",
    );

    // ...and from here on the window is not moving, so a per-frame recompute would
    // be visible as the value being rewritten on every single frame. Scrambling it
    // in between and checking it survived pins the guard.
    let mut scrambled = app
        .world_mut()
        .entity_mut(node_entity)
        .into_mut::<Node>()
        .unwrap();
    scrambled.width = Val::Px(7.0);
    app.update();
    app.update();

    assert_eq!(
        px(read::<Node>(&mut app, node_entity).width, "steady frames"),
        7.0,
        "the system recomputed on frames where the window had not changed",
    );
}
/// The rescale must not run on a frame where nothing about the window changed.
///
/// This is the guard, and it is load-bearing rather than an optimisation.
/// `update_ui_scale` assigns through `ResMut`, which marks the resource changed
/// *every frame* - so an unconditional assignment makes `is_changed()` permanently
/// true, the rescale runs sixty times a second, and it overwrites the size
/// `buttons::apply_visual` just applied. The symptom was a hovered button growing
/// for a moment and snapping back while its colour stayed green, because the
/// rescale owns the width and nothing else does.
///
/// Written against `UiScale` directly rather than against a window: what is under
/// test is the change-detection signal the rescale keys on.
#[test]
fn an_unchanged_scale_does_not_touch_a_node() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(UiScale(1.0));
    app.add_systems(Update, rescale_ui_system);

    // Painted with a deliberately wrong size: the rescale corrects it the first
    // time it runs and must leave it alone afterwards.
    let node_entity = spawn_scaled(&mut app, ScaledNode::sized(150.0, 50.0), Vec2::new(1.0, 1.0));
    app.update();
    assert_eq!(
        px(read::<Node>(&mut app, node_entity).width, "first frame"),
        150.0,
        "the first frame should apply the scale",
    );

    // Now scramble it. If the rescale ran on an idle frame it would put 150.0 back
    // and this would pass for the wrong reason; here it has to stay scrambled.
    app.world_mut().entity_mut(node_entity).into_mut::<Node>().unwrap().width = Val::Px(7.0);
    app.update();
    app.update();

    assert_eq!(
        px(read::<Node>(&mut app, node_entity).width, "idle frames"),
        7.0,
        "the rescale rewrote the UI on frames where the window had not changed, so \
         it fights the hover handling over the size of a button",
    );
}

/// And the visible consequence: a hovered button keeps its hovered size when the
/// scale is applied afterwards.
///
/// The two systems have to agree. `rescale_ui_system` replays `ScaledNode`, and
/// `apply_visual` records the hovered size into it, so a resize during a hover
/// restores the hovered size rather than the resting one.
#[test]
fn a_resize_keeps_a_hovered_button_enlarged() {
    use crate::buttons::{BUTTON_HOVERED_SIZE, BUTTON_SIZE};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(UiScale(1.0));
    app.add_systems(Update, rescale_ui_system);

    let button = app
        .world_mut()
        .spawn((
            ScaledNode::sized(BUTTON_SIZE.x, BUTTON_SIZE.y),
            Node {
                width: Val::Px(BUTTON_SIZE.x),
                height: Val::Px(BUTTON_SIZE.y),
                ..default()
            },
        ))
        .id();
    app.update();

    // What `apply_visual` does on hover: paint the enlarged size and record it in
    // design space so the rescale can replay it.
    {
        let mut entity = app.world_mut().entity_mut(button);
        *entity.get_mut::<Node>().unwrap() = Node {
            width: Val::Px(BUTTON_HOVERED_SIZE.x),
            height: Val::Px(BUTTON_HOVERED_SIZE.y),
            ..default()
        };
        entity.get_mut::<ScaledNode>().unwrap().size = Some(BUTTON_HOVERED_SIZE);
    }

    // The window doubles.
    app.insert_resource(UiScale(2.0));
    app.update();
    let node = app.world().entity(button).get::<Node>().unwrap().clone();

    assert_eq!(
        px(node.width, "hovered after resize"),
        BUTTON_HOVERED_SIZE.x * 2.0,
        "the resize restored the resting size on a button that is still hovered",
    );
}

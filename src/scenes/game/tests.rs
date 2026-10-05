//! Tests for the room carousel.
//!
//! The modulo here is wrapped arithmetic in both directions, which is the
//! classic place for an off-by-one to hide: a room with one variant must not
//! move, and an empty room must not divide by zero.

use bevy::prelude::*;

use super::systems::{item_hotspot_visibility_system, step};
use crate::acts::{ActId, CurrentAct, Item};
use crate::scenes::game::items::WorldItems;
use crate::scenes::game::rooms::components::{
    HOTSPOT_OUTLINE_THICKNESS, Hotspot, HotspotDef, HotspotOutline,
};
use crate::scenes::game::rooms::data::{p, room_def};
use crate::scenes::game::ui::CarouselDir;

/// Bevy validates a system's queries when the system first runs, and panics
/// with B0001 if two of its parameters write the same component without being
/// declared disjoint. That panic happens at launch, not at build time, so
/// nothing else in `cargo test` would notice it: `item_hotspot_visibility_system`
/// hides both a hotspot and the chevron beside it, and both are `Visibility`.
///
/// Running the system in a bare app is the whole test.
#[test]
fn the_item_hotspot_system_has_no_conflicting_queries() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(WorldItems::with_resting([(
            Item::Teddy,
            "tex/rooms/floor_2/ap_1.png",
        )]))
        .add_systems(Update, item_hotspot_visibility_system);
    app.update();
}

#[test]
fn single_variant_has_nowhere_to_go() {
    assert_eq!(step(0, 1, CarouselDir::Next), None);
    assert_eq!(step(0, 1, CarouselDir::Prev), None);
}

/// The room would have no variants at all. The modulo below divides by `count`,
/// so this has to be refused rather than panicked on.
#[test]
fn empty_room_does_not_divide_by_zero() {
    assert_eq!(step(0, 0, CarouselDir::Next), None);
    assert_eq!(step(0, 0, CarouselDir::Prev), None);
}

#[test]
fn next_moves_forward() {
    assert_eq!(step(0, 2, CarouselDir::Next), Some(1));
    assert_eq!(step(1, 3, CarouselDir::Next), Some(2));
}

#[test]
fn prev_moves_backward() {
    assert_eq!(step(1, 2, CarouselDir::Prev), Some(0));
    assert_eq!(step(2, 3, CarouselDir::Prev), Some(1));
}

/// The ends wrap around rather than sticking or going out of bounds.
#[test]
fn ends_wrap() {
    assert_eq!(
        step(1, 2, CarouselDir::Next),
        Some(0),
        "last wraps to first"
    );
    assert_eq!(
        step(0, 3, CarouselDir::Prev),
        Some(2),
        "first wraps to last"
    );
}

#[test]
fn result_is_always_in_range() {
    for count in 2..8 {
        for index in 0..count {
            for dir in [CarouselDir::Next, CarouselDir::Prev] {
                let next = step(index, count, dir).expect("a move is available");
                assert!(
                    next < count,
                    "count {count} index {index} {dir:?} produced {next}",
                );
                assert_ne!(next, index, "a step must change the index");
            }
        }
    }
}

/// Pressing one arrow repeatedly must visit every variant and return to the
/// start, for any number of variants.
#[test]
fn cycling_visits_every_variant() {
    for count in 2..8 {
        for dir in [CarouselDir::Next, CarouselDir::Prev] {
            let mut seen = vec![false; count];
            let mut index = 0;
            seen[0] = true;
            for _ in 0..count - 1 {
                index = step(index, count, dir).expect("a move is available");
                assert!(!seen[index], "count {count} {dir:?} revisited {index}");
                seen[index] = true;
            }
            assert!(
                seen.iter().all(|hit| *hit),
                "count {count} {dir:?} skipped a variant"
            );
            assert_eq!(
                step(index, count, dir),
                Some(0),
                "count {count} {dir:?} did not come full circle",
            );
        }
    }
}

/// The `impl Default` blocks became `#[derive(Default)]` with `#[default]` on a
/// variant. Same variant has to come out, or the game quietly starts in the wrong
/// room / plays the wrong door sound.
#[test]
fn derived_defaults_match_what_the_manual_impls_did() {
    use crate::acts::ActId;
    use crate::scenes::sound::TransitionSound;

    assert_eq!(ActId::default(), ActId::ActOne);
    assert_eq!(TransitionSound::default(), TransitionSound::NextRoom);
}

// ------------------------------------------------- the backdrop stays behind

/// The blurred backdrop is spawned *after* the room, and the room picture is a
/// child of `Room` drawing at z 0. Two sprites on the same z are ordered by when
/// they were spawned, so the backdrop wins the tie and goes over the room: the
/// player sees `main_fon.png` with a working inventory and clickable hotspots on
/// top, and no room.
///
/// It went unnoticed because the menu and the intro both spawn their own backdrop
/// and despawn it on the way out, so a room and a backdrop had never been drawn
/// at the same z before. `--rooms` reaches `OnEnter(Game)` directly, which is
/// where it showed up.
///
/// Asserted against the room picture's own z rather than a constant, because the
/// room is what has to be visible: read as two literals, the test would pass
/// whatever both happened to be.
#[test]
fn the_backdrop_is_behind_the_room_picture() {
    use crate::acts::{ActId, CurrentAct};
    use crate::scenes::game::rooms::components::Room;
    use crate::scenes::game::rooms::data::{p, room_def};
    use crate::scenes::game::rooms::spawn::spawn_room;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .insert_resource(CurrentAct(ActId::ActOne));

    // The real spawner rather than a hand-built entity, so the z this reads is
    // the one the game draws with.
    let def = room_def(p::F1_STREET_1, ActId::ActOne);
    app.add_systems(
        Update,
        move |mut commands: Commands, asset_server: Res<AssetServer>| {
            spawn_room(&mut commands, &asset_server, def, Vec3::ZERO);
        },
    );
    app.update();

    let world = app.world_mut();
    // The room's own transform is the picture's: the picture is its first child
    // and nothing moves a child in z, so the root carries the whole room's layer.
    let mut rooms = world.query_filtered::<&Transform, With<Room>>();
    let room_picture = rooms
        .iter(world)
        .next()
        .expect("the room spawned")
        .translation
        .z;

    assert!(
        super::ui::BACKDROP_Z < room_picture,
        "the backdrop is at z {} and the room picture at z {room_picture}, so the \\
         backdrop is drawn over the room",
        super::ui::BACKDROP_Z,
    );
}

// ------------------------------------------------------- the hover outline

/// Every hotspot gets four outline bars, hidden until hovered.
///
/// The count is four because they are drawn that way, and the count is what a bug
/// here shows up as: a frame with three sides, or two bars left showing where a
/// door used to be after the carousel moved on.
#[test]
fn every_hotspot_has_four_outline_bars() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .insert_resource(CurrentAct(ActId::ActOne));

    // The real spawner, not a hand-built `Room`: a hand-built one has no
    // hotspots and no outlines, so every assertion below would pass on an empty
    // world rather than on the thing being tested.
    let def = room_def(p::F1_STREET_1, ActId::ActOne);
    app.add_systems(
        Update,
        move |mut commands: Commands, asset_server: Res<AssetServer>| {
            crate::scenes::game::rooms::spawn::spawn_room(
                &mut commands,
                &asset_server,
                def,
                Vec3::ZERO,
            );
        },
    );
    app.update();

    let world = app.world_mut();
    let mut hotspots = world.query_filtered::<Entity, With<Hotspot>>();
    let doors: Vec<Entity> = hotspots.iter(world).collect();
    assert!(!doors.is_empty(), "the room spawned no hotspots");

    let mut bars = world.query_filtered::<Entity, With<HotspotOutline>>();
    for door in doors {
        let count = bars
            .iter(world)
            .filter(|bar| world.get::<ChildOf>(*bar).map(|c| c.parent()) == Some(door))
            .count();
        assert_eq!(count, 4, "a hotspot does not have four outline bars");
    }
}

/// The bars start hidden.
///
/// Not merely transparent: `Visibility::Hidden` also takes them out of the
/// picking backend, so a bar left visible by a bug cannot intercept a click meant
/// for the door underneath it.
#[test]
fn outline_bars_start_hidden() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .insert_resource(CurrentAct(ActId::ActOne));

    // The real spawner, not a hand-built `Room`: a hand-built one has no
    // hotspots and no outlines, so every assertion below would pass on an empty
    // world rather than on the thing being tested.
    let def = room_def(p::F1_STREET_1, ActId::ActOne);
    app.add_systems(
        Update,
        move |mut commands: Commands, asset_server: Res<AssetServer>| {
            crate::scenes::game::rooms::spawn::spawn_room(
                &mut commands,
                &asset_server,
                def,
                Vec3::ZERO,
            );
        },
    );
    app.update();

    let world = app.world_mut();
    let mut bars = world.query::<(&HotspotOutline, &Visibility)>();
    let seen: Vec<Visibility> = bars
        .iter(world)
        .map(|(_, visibility)| *visibility)
        .collect();
    assert!(!seen.is_empty(), "no outline bars spawned");
    for visibility in seen {
        assert_eq!(
            visibility,
            &Visibility::Hidden,
            "an outline bar is visible before anything is hovered",
        );
    }
}

/// The bars must never be pickable.
///
/// A pickable bar sits exactly on top of the door, and picking hands a click to the
/// topmost entity at a point. If the bars could be picked, clicking a door could
/// land on a bar - which carries no `HotspotAction`, so the click would be read by
/// `game_hotspot_system` and dropped, and the player would find doors that sometimes
/// work and sometimes do not.
#[test]
fn outline_bars_are_not_pickable() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .insert_resource(CurrentAct(ActId::ActOne));

    // The real spawner, not a hand-built `Room`: a hand-built one has no
    // hotspots and no outlines, so every assertion below would pass on an empty
    // world rather than on the thing being tested.
    let def = room_def(p::F1_STREET_1, ActId::ActOne);
    app.add_systems(
        Update,
        move |mut commands: Commands, asset_server: Res<AssetServer>| {
            crate::scenes::game::rooms::spawn::spawn_room(
                &mut commands,
                &asset_server,
                def,
                Vec3::ZERO,
            );
        },
    );
    app.update();

    let world = app.world_mut();
    let mut bars = world.query::<(&HotspotOutline, &Pickable)>();
    let pickable: Vec<Pickable> = bars.iter(world).map(|(_, pickable)| *pickable).collect();
    assert!(!pickable.is_empty(), "no outline bars spawned");
    for pickable in pickable {
        assert!(
            !pickable.should_block_lower,
            "an outline bar can take a click meant for the door",
        );
        assert!(
            !pickable.is_hoverable,
            "an outline bar can be hovered, which is not how the outline is driven",
        );
    }
}

/// The outline is drawn above its hotspot, but below an NPC's click target.
///
/// The same reasoning as `her_target_sits_above_the_door_behind_her`, one layer
/// down. The concierge's way out is a hotspot, the granny stands in front of it, and
/// an outline drawn over her target would cover the part of the door she overlaps -
/// so the player would click what looks like the doorway and, while the pointer was
/// over her, talk to her instead of leaving the room.
///
/// Read out of spawned entities rather than compared as constants: both numbers are
/// literals in the source, so comparing them would pass whatever they happened to
/// be and prove nothing about what is drawn on top of what.
#[test]
fn the_outline_is_drawn_above_its_hotspot() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .insert_resource(CurrentAct(ActId::ActOne));

    let def = room_def(p::F1_STREET_1, ActId::ActOne);
    app.add_systems(
        Update,
        move |mut commands: Commands, asset_server: Res<AssetServer>| {
            crate::scenes::game::rooms::spawn::spawn_room(
                &mut commands,
                &asset_server,
                def,
                Vec3::ZERO,
            );
        },
    );
    app.update();

    let world = app.world_mut();
    let mut doors = world.query_filtered::<(Entity, &Transform), With<Hotspot>>();
    let mut outlines = world.query_filtered::<(Entity, &Transform), With<HotspotOutline>>();

    let mut checked = 0;
    for (hotspot, hotspot_transform) in doors.iter(world) {
        // World z is the sum along the parent chain: the hotspot is a child of the
        // room, and the bars are children of the hotspot.
        let hotspot_z = world
            .get::<ChildOf>(hotspot)
            .map(|child| world.get::<Transform>(child.parent()).map_or(0.0, |t| t.translation.z))
            .unwrap_or(0.0)
            + hotspot_transform.translation.z;

        for (outline, outline_transform) in outlines.iter(world) {
            let parent = world.get::<ChildOf>(outline).map(|child| child.parent());
            if parent != Some(hotspot) {
                continue;
            }
            checked += 1;
            assert!(
                hotspot_z + outline_transform.translation.z > hotspot_z,
                "the outline is drawn at or below the hotspot it belongs to",
            );
        }
    }

    assert!(checked > 0, "no outline bars were found above a hotspot");
}

/// And below an NPC's click target, which is the layering that keeps the concierge
/// reachable. Compared against the real `NPC_TARGET_Z` through the test-only
/// re-export, so a change there cannot quietly invalidate the order.
#[test]
fn the_outline_is_drawn_below_an_npcs_click_target() {
    let npc_target_z = super::npc::NPC_TARGET_Z_FOR_TESTS;
    let outline_z = {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Image>()
            .insert_resource(CurrentAct(ActId::ActOne));

        let def = room_def(p::F1_STREET_1, ActId::ActOne);
        app.add_systems(
            Update,
            move |mut commands: Commands, asset_server: Res<AssetServer>| {
                crate::scenes::game::rooms::spawn::spawn_room(
                    &mut commands,
                    &asset_server,
                    def,
                    Vec3::ZERO,
                );
            },
        );
        app.update();

        let world = app.world_mut();
        let mut hotspots =
            world.query_filtered::<&Transform, (With<Hotspot>, Without<HotspotOutline>)>();
        let hotspot_z = hotspots
            .iter(world)
            .next()
            .expect("the room spawned a hotspot")
            .translation
            .z;

        let mut bars =
            world.query_filtered::<&Transform, (With<HotspotOutline>, Without<Hotspot>)>();
        hotspot_z
            + bars
                .iter(world)
                .next()
                .expect("the room spawned an outline")
                .translation
                .z
    };

    assert!(
        outline_z < npc_target_z,
        "the outline is at world z {outline_z} and an NPC's click target at \
         {npc_target_z}, so the outline covers the character standing in front of \
         the door",
    );
}

/// The pulse stays inside the range the constants say it does.
///
/// A sine mapped badly overshoots into a negative alpha, which wgpu treats as
/// undefined rather than as transparent, so the bars would flicker to garbage
/// instead of dimming.
#[test]
fn the_outline_pulse_stays_in_range() {
    for step in 0..2000 {
        let t = step as f32 * 0.01;
        for fade in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let alpha = super::systems::outline_alpha(t, fade);
            assert!(
                (0.0..=1.0).contains(&alpha),
                "alpha {alpha} is outside 0..1 at t={t} fade={fade}",
            );
            assert!(
                alpha <= fade + f32::EPSILON,
                "alpha {alpha} is brighter than its fade {fade} allows",
            );
        }
    }
}

/// The four bars have to close into a frame around the hotspot.
///
/// This is the test for "the outline does not go all the way round". Each bar is
/// checked against the hotspot's own edges: the horizontal pair has to span the
/// full width and sit on the top and bottom edges, the vertical pair the full
/// height on the left and right. A bar that is the right length but on the wrong
/// edge draws a frame with a gap in it, which reads as a deliberate partial
/// highlight rather than as a bug.
#[test]
fn the_outline_bars_close_into_a_frame() {
    const EPS: f32 = 0.01;

    for (name, size) in [
        ("door", Vec2::new(200.0, 300.0)),
        ("wide door", Vec2::new(400.0, 400.0)),
        ("item spot", Vec2::new(140.0, 140.0)),
        ("tall exit", Vec2::new(150.0, 610.0)),
        ("very small", Vec2::new(10.0, 10.0)),
    ] {
        let t = HOTSPOT_OUTLINE_THICKNESS;
        let (half_w, half_h) = (size.x / 2.0, size.y / 2.0);
        // Mirrors `spawn_outline`: full length, never clamped to the thickness.
        let bar_w = size.x + t;
        let bar_h = size.y + t;

        let horizontal = [
            (Vec2::new(0.0, half_h), Vec2::new(bar_w, t)),
            (Vec2::new(0.0, -half_h), Vec2::new(bar_w, t)),
        ];
        let vertical = [
            (Vec2::new(half_w, 0.0), Vec2::new(t, bar_h)),
            (Vec2::new(-half_w, 0.0), Vec2::new(t, bar_h)),
        ];

        // The horizontal pair spans the width and lies on the y edges.
        for (at, extent) in horizontal {
            assert!(
                (at.y.abs() - half_h).abs() < EPS,
                "{name}: a horizontal bar sits at y={} instead of on the edge \
                 at y={half_h}",
                at.y,
            );
            assert!(
                extent.x >= size.x,
                "{name}: a horizontal bar is {} wide and the hotspot is {}, so \
                 the frame has a gap in it",
                extent.x,
                size.x,
            );
            assert!(
                (extent.y - t).abs() < EPS,
                "{name}: a horizontal bar is {} thick, expected {t}",
                extent.y,
            );
        }

        // The vertical pair spans the height and lies on the x edges.
        for (at, extent) in vertical {
            assert!(
                (at.x.abs() - half_w).abs() < EPS,
                "{name}: a vertical bar sits at x={} instead of on the edge at \
                 x={half_w}",
                at.x,
            );
            assert!(
                extent.y >= size.y,
                "{name}: a vertical bar is {} tall and the hotspot is {}, so the \
                 frame has a gap in it",
                extent.y,
                size.y,
            );
            assert!(
                (extent.x - t).abs() < EPS,
                "{name}: a vertical bar is {} thick, expected {t}",
                extent.x,
            );
        }

        // The corners have to be covered by one or the other. The horizontal bars
        // run the full width and the vertical bars the full height, so they
        // overlap at all four corners - checked here so that a later change to
        // either one cannot quietly open a corner.
        let corners = [
            Vec2::new(half_w, half_h),
            Vec2::new(-half_w, half_h),
            Vec2::new(half_w, -half_h),
            Vec2::new(-half_w, -half_h),
        ];
        for corner in corners {
            let by_horizontal = horizontal.iter().any(|(at, extent)| {
                (corner.y - at.y).abs() <= extent.y / 2.0 + EPS
                    && corner.x.abs() <= extent.x / 2.0 + EPS
            });
            let by_vertical = vertical.iter().any(|(at, extent)| {
                (corner.x - at.x).abs() <= extent.x / 2.0 + EPS
                    && corner.y.abs() <= extent.y / 2.0 + EPS
            });
            assert!(
                by_horizontal || by_vertical,
                "{name}: the corner at {corner:?} is covered by neither bar, so the \
                 outline has a hole there",
            );
        }
    }
}

/// The bars that actually spawn are the four edges of the hotspot, measured on a
/// real room rather than on the arithmetic above.
///
/// The arithmetic test pins the layout; this one pins that `spawn_room_content`
/// uses it. They are different claims, and the bug this replaced was in the
/// spawner - it clamped each bar's length to four thicknesses, so a real door got
/// a 20px dash on each edge instead of a frame, while a test written against the
/// intended numbers would still have passed.
#[test]
fn the_spawned_bars_cover_the_hotspot_edges() {
    const EPS: f32 = 0.5;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .insert_resource(CurrentAct(ActId::ActOne));

    let def = room_def(p::F1_STREET_1, ActId::ActOne);
    app.add_systems(
        Update,
        move |mut commands: Commands, asset_server: Res<AssetServer>| {
            crate::scenes::game::rooms::spawn::spawn_room(
                &mut commands,
                &asset_server,
                def,
                Vec3::ZERO,
            );
        },
    );
    app.update();

    let world = app.world_mut();
    let mut hotspots = world.query_filtered::<(Entity, &HotspotDef), With<Hotspot>>();
    let doors: Vec<(Entity, Vec2)> = hotspots
        .iter(world)
        .map(|(entity, hot)| (entity, hot.size))
        .collect();
    assert!(!doors.is_empty(), "no hotspots spawned");

    let mut bars = world.query::<(&ChildOf, &Transform, &Sprite)>();
    let all: Vec<(Entity, Vec2, Vec2)> = bars
        .iter(world)
        .map(|(parent, transform, sprite)| {
            (
                parent.parent(),
                transform.translation.truncate(),
                sprite.custom_size.unwrap_or_default(),
            )
        })
        .collect();

    for (hotspot, size) in doors {
        let mine: Vec<&(Entity, Vec2, Vec2)> =
            all.iter().filter(|(owner, ..)| *owner == hotspot).collect();
        assert_eq!(mine.len(), 4, "a hotspot has {} bars", mine.len());

        let (half_w, half_h) = (size.x / 2.0, size.y / 2.0);
        let widest = mine
            .iter()
            .map(|(_, _, extent)| extent.x)
            .fold(0.0_f32, f32::max);
        let tallest = mine
            .iter()
            .map(|(_, _, extent)| extent.y)
            .fold(0.0_f32, f32::max);

        assert!(
            widest >= size.x - EPS && tallest >= size.y - EPS,
            "the bars cover at most {widest}x{tallest} for a {size:?} hotspot, so \
             they cannot close into a frame",
        );

        // And every edge must have a bar on it: no run of edge longer than the
        // thickness may be without one, which is what a clamped bar produces.
        let on_top = mine.iter().any(|(_, at, extent)| {
            (at.y - half_h).abs() <= EPS && extent.x >= size.x - EPS
        });
        let on_bottom = mine.iter().any(|(_, at, extent)| {
            (at.y + half_h).abs() <= EPS && extent.x >= size.x - EPS
        });
        let on_left = mine
            .iter()
            .any(|(_, at, extent)| (at.x + half_w).abs() <= EPS && extent.y >= size.y - EPS);
        let on_right = mine
            .iter()
            .any(|(_, at, extent)| (at.x - half_w).abs() <= EPS && extent.y >= size.y - EPS);

        assert!(
            on_top && on_bottom && on_left && on_right,
            "a {size:?} hotspot is missing an edge: top={on_top} bottom={on_bottom} \
             left={on_left} right={on_right}",
        );
    }
}

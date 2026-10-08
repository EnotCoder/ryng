//! Tests for the room carousel.
//!
//! The modulo here is wrapped arithmetic in both directions, which is the
//! classic place for an off-by-one to hide: a room with one variant must not
//! move, and an empty room must not divide by zero.

use std::collections::HashSet;

use bevy::prelude::*;

use super::systems::{
    carousel_control_system, carousel_system, item_hotspot_visibility_system, room_flip_system,
};
use crate::acts::{ActId, CurrentAct, Item};
use crate::scenes::game::items::WorldItems;
use crate::scenes::game::rooms::components::{
    ANIM_FRAME_SECONDS, HOTSPOT_OUTLINE_THICKNESS, Hotspot, HotspotDef, HotspotOutline, Room,
    RoomAnim, RoomFlip, RoomTitle, RoomVariantIndex,
};
use crate::scenes::game::rooms::data::{RoomDef, all_paths, control_target, p, room_def, rooms};
use crate::scenes::game::ui::{CarouselArrow, spawn_game_ui};
use crate::scenes::fade::{FADE_DURATION, RoomFade};
use crate::scenes::loading::PreloadedImages;
use crate::scenes::game::StartRoom;
use crate::UiScale;

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
    assert_eq!(control_target(0, 1), None);
}

/// The room would have no variants at all. The modulo below divides by `count`,
/// so this has to be refused rather than panicked on.
#[test]
fn empty_room_does_not_divide_by_zero() {
    assert_eq!(control_target(0, 0), None);
}

/// With the two shots every carousel in the game has, the one control leads to the
/// other shot.
#[test]
fn the_control_leads_to_the_other_shot() {
    assert_eq!(control_target(0, 2), Some(1), "the lift offers the stairs");
    assert_eq!(control_target(1, 2), Some(0), "the stairs offer the lift");
}

/// The ends wrap around rather than sticking or going out of bounds.
#[test]
fn ends_wrap() {
    assert_eq!(control_target(1, 2), Some(0), "last wraps to first");
}

#[test]
fn result_is_always_in_range() {
    for count in 2..8 {
        for index in 0..count {
            let next = control_target(index, count).expect("a move is available");
            assert!(next < count, "count {count} index {index} produced {next}");
            assert_ne!(next, index, "a step must change the index");
        }
    }
}

/// Pressing the control repeatedly must visit every shot and come full circle, for
/// any number of shots.
///
/// There is no "back" any more - the control always moves the same way - so this is
/// also what pins that a carousel of any length can be walked from end to end.
#[test]
fn cycling_visits_every_variant() {
    for count in 2..8 {
        let mut seen = vec![false; count];
        let mut index = 0;
        seen[0] = true;
        for _ in 0..count - 1 {
            index = control_target(index, count).expect("a move is available");
            assert!(!seen[index], "count {count} revisited {index}");
            seen[index] = true;
        }
        assert!(
            seen.iter().all(|hit| *hit),
            "count {count} skipped a variant"
        );
        assert_eq!(
            control_target(index, count),
            Some(0),
            "count {count} did not come full circle",
        );
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

// -------------------------------------------------------------- the pan between shots

/// Three frames standing in for the twelve, so a failure names a short list.
const PAN: &[&str] = &["a/1.png", "a/2.png", "a/3.png"];

/// The frames a room flips through, in order.
///
/// Takes the row by value: `RoomDef` is `Copy` and `room_def` hands one back
/// rather than a reference into the table.
fn pan_of(room: RoomDef) -> Vec<&'static str> {
    room.flip.expect("this room has a pan").to_vec()
}

/// Every frame of a pan, in the order the player would see them.
///
/// The first frame is on screen before `advance` is ever called, so the walk starts
/// there. `advance` returns false once the pan is over, which is what ends the loop
/// - not the count.
fn frames_seen(mut flip: RoomFlip) -> Vec<&'static str> {
    let mut seen = vec![flip.frame()];
    while flip.advance() {
        seen.push(flip.frame());
    }
    seen
}

/// A pan must show every frame exactly once, in the order it was drawn.
///
/// The off-by-one this pins is the last frame: `advance` answers "is there another
/// one after this", so a version that landed on the tick which *showed* the final
/// frame would finish without ever displaying it, and the pan would stop one step
/// short of the stairs.
#[test]
fn a_pan_shows_every_frame_once_in_order() {
    let seen = frames_seen(RoomFlip::new(PAN, 1, true));

    assert_eq!(
        seen, PAN,
        "the pan skipped or repeated a frame, so the room jumps mid-pan",
    );
}

/// Going the other way is the same list read backwards.
///
/// One list serves both arrows, so the reverse pass is a second walk over the same
/// array rather than a second set of files. Compared against the forward walk
/// reversed, which also catches an index that runs off the front of the array - the
/// backwards walk is the one that can go negative.
#[test]
fn a_pan_backwards_is_the_same_frames_in_reverse() {
    let forwards = frames_seen(RoomFlip::new(PAN, 1, true));
    let backwards = frames_seen(RoomFlip::new(PAN, 0, false));
    let mut expected = forwards;
    expected.reverse();

    assert_eq!(
        backwards, expected,
        "the two directions do not show the same frames, so the pan jumps on the way back",
    );
    assert_eq!(
        backwards.first(),
        Some(&PAN[PAN.len() - 1]),
        "a pan played backwards must start on the frame the art ends on",
    );
}

/// An animation must show every frame exactly once, in the order it was drawn.
///
/// The same off-by-one the pan has, and for the same reason: `advance` answers
/// "is there another one after this", so landing on the tick that *showed* the
/// final frame would finish without ever displaying it, and the fall would stop
/// one step short of the dark the player is meant to end in.
#[test]
fn an_animation_shows_every_frame_once_in_order() {
    let def = room_def(p::ELEVATOR, ActId::ActOne);
    let frames = def.anim.expect("the lift plays an animation");
    let mut anim = RoomAnim::new(frames);

    let mut seen = vec![anim.frame()];
    while anim.advance() {
        seen.push(anim.frame());
    }

    assert_eq!(
        seen, frames,
        "the animation skipped or repeated a frame, so the fall jumps",
    );
}

/// Once the frames are done the animation stops rather than running off the end.
///
/// A pan can stop because the player arrived somewhere; an animation has nowhere
/// to arrive, so it just runs out. What must not happen is an index past the last
/// frame, which is a panic on the frame after the fall finishes - and the system
/// drops the component there, so this is the last thing it ever reads.
#[test]
fn a_finished_animation_stays_on_its_last_frame() {
    let def = room_def(p::ELEVATOR, ActId::ActOne);
    let frames = def.anim.expect("the lift plays an animation");
    let mut anim = RoomAnim::new(frames);

    while anim.advance() {}

    assert_eq!(
        anim.frame(),
        frames[frames.len() - 1],
        "a spent animation reads past its own frames",
    );
}

/// The lift is a beat: a room that plays itself and then hands over on its own.
///
/// Worth pinning because the two halves are what make the fall work at all. It has
/// to be a beat, or there is no `auto_next` and the player is stranded looking at
/// the dark; and the frames have to be longer than the sound that goes with them,
/// or the animation finishes under the player and the last second of the fall is
/// a still picture.
#[test]
fn the_lift_falls_for_longer_than_its_sound() {
    let def = room_def(p::ELEVATOR, ActId::ActOne);
    let frames = def.anim.expect("the lift plays an animation");

    assert!(!def.interactive, "the lift is a beat, so nothing is clickable");
    let (target, seconds) = def.auto_next.expect("the lift moves on by itself");
    assert_eq!(target, p::B_HALL, "the lift lands in the basement");

    // The room is spawned inside a fade-in and leaves inside a fade-out, and the
    // sound is killed when it ends, so the room is on screen for its `auto_next`
    // plus a fade at each end. The animation has to fill that, not just the
    // `auto_next` part of it.
    let on_screen = frames.len() as f32 * ANIM_FRAME_SECONDS;
    let visible = seconds + 2.0 * FADE_DURATION;
    assert!(
        on_screen > visible,
        "the fall is over in {on_screen}s but the room stands for {visible}s, so the \
         last {}s are a still picture",
        visible - on_screen,
    );
}

/// The destination is fixed when the pan starts, so the room lands where the player
/// asked to go rather than where the frames happen to end.
#[test]
fn a_pan_keeps_where_it_was_going() {
    let flip = RoomFlip::new(PAN, 1, true);

    assert_eq!(flip.to, 1, "the destination moved while the pan was playing");
}

// ------------------------------------------------------------ the table names them

/// Both halls pan, and neither borrows the other's frames.
///
/// The two visits to this corridor are not the same moment: the first time the lift
/// works and the second time it does not. Two folders exist for that reason, and
/// the table is the only thing that keeps them apart - a room built from the same
/// list twice would show a clean elevator door to a player who has already watched
/// it fall, which is the whole ending of act 1.
#[test]
fn each_hall_flips_with_its_own_pan() {
    let working = room_def(p::F1_HALL, ActId::ActOne);
    let dead = room_def(p::F1_HALL_DEAD, ActId::ActOne);

    let working_frames = pan_of(working);
    let dead_frames = pan_of(dead);

    assert!(
        !working_frames.is_empty() && !dead_frames.is_empty(),
        "a hall with no frames cuts instead of panning",
    );
    assert!(
        working_frames.iter().all(|frame| !dead_frames.contains(frame)),
        "the two halls share frames: {:?}",
        working_frames
            .iter()
            .filter(|frame| dead_frames.contains(frame))
            .collect::<Vec<_>>(),
    );
}

/// The same number of frames both times, so the corridor does not change pace
/// between visits.
///
/// The two pans are drawn to the same timing, and they only differ in the door. A
/// different frame count in one of them means the art was re-exported and the
/// difference is invisible here and very visible in the game.
#[test]
fn both_halls_pan_over_the_same_number_of_frames() {
    let working = pan_of(room_def(p::F1_HALL, ActId::ActOne));
    let dead = pan_of(room_def(p::F1_HALL_DEAD, ActId::ActOne));

    assert_eq!(
        working.len(),
        dead.len(),
        "one hall's pan is {} frames and the other's is {}, so the corridor changes \
         speed between the two visits",
        working.len(),
        dead.len(),
    );
}

/// Every frame of a pan comes from one folder.
///
/// The list is twelve literal paths written by hand, so a mistyped frame is the
/// obvious mistake - and it would show up as one hard frame in the middle of a
/// smooth pan rather than as anything a build would notice.
#[test]
fn a_pan_is_played_from_one_folder() {
    for room in [p::F1_HALL, p::F1_HALL_DEAD] {
        let def = room_def(room, ActId::ActOne);
        let folders: HashSet<_> = pan_of(def)
            .iter()
            .map(|frame| frame.rsplit_once('/').map(|(dir, _)| dir))
            .collect();

        assert_eq!(
            folders.len(),
            1,
            "the pan for {room} mixes folders: {:?}",
            folders,
        );
    }
}

/// Only the two halls have a pan, and every other room still cuts.
///
/// A room with a single shot has nothing to pan between: given frames it would play
/// them and land back where it started, which is a half-second of blurred corridor
/// for no reason at all.
#[test]
fn only_a_room_with_shots_to_flip_has_a_pan() {
    let mut with_pans = Vec::new();
    for room in rooms() {
        let Some(frames) = room.flip else {
            continue;
        };
        with_pans.push(room.variants[0].path);
        assert!(
            !frames.is_empty(),
            "{} has a pan with no frames in it",
            room.variants[0].path,
        );
        assert!(
            room.variants.len() > 1,
            "{} has {} shot but a pan, so the pan lands back on the shot it started from",
            room.variants[0].path,
            room.variants.len(),
        );
    }

    with_pans.sort();
    assert_eq!(
        with_pans,
        vec![p::F1_HALL, p::F1_HALL_DEAD],
        "a room gained or lost its pan",
    );
}

/// The pan ends on the shot it was heading for, and hands the room back.
///
/// The component tests above pin which frames play; this one pins that the system
/// puts them on the room and then takes them off again. Run against the real
/// spawner and the real hall row, because the claim is about a room in a game and
/// not about an array - a pan that played every frame and never landed would
/// satisfy all of them.
#[test]
fn a_played_pan_lands_on_the_shot_and_clears_itself() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .insert_resource(CurrentAct(ActId::ActOne));

    let def = room_def(p::F1_HALL, ActId::ActOne);
    let frames = pan_of(def);
    let to = control_target(0, def.variants.len()).expect("the hall has a second shot");

    // The spawner runs once and only once: run it every update and it puts a second
    // room in the world, and every system that reads "the room" stops matching when
    // there is more than one - which is a silent no-op, not a failure.
    app.add_systems(
        Update,
        move |mut commands: Commands, asset_server: Res<AssetServer>, mut spawned: Local<bool>| {
            if *spawned {
                return;
            }
            *spawned = true;
            crate::scenes::game::rooms::spawn::spawn_room(
                &mut commands,
                &asset_server,
                def,
                Vec3::ZERO,
            );
        },
    );
    app.add_systems(Update, room_flip_system);
    app.update();

    // Stand the pan up the way the carousel system does: the first frame on the room
    // and the component naming where it is going.
    let world = app.world_mut();
    let room = world
        .query_filtered::<Entity, With<Room>>()
        .iter(world)
        .next()
        .expect("the hall spawned");
    let first = world.spawn(Sprite::from_image(Handle::<Image>::default())).id();
    world.entity_mut(first).set_parent_in_place(room);
    world
        .entity_mut(room)
        .insert(RoomFlip::new(def.flip.unwrap(), to, true));

    // Run the frames out. The timer is zeroed rather than the wall clock waited on:
    // the pan is twelve frames at a twentieth of a second, and a test that sleeps for
    // them is a test that fails on a loaded machine.
    for step in 0..frames.len() + 1 {
        app.update();
        if step == frames.len() {
            break;
        }
        let mut pans = app
            .world_mut()
            .query_filtered::<&mut RoomFlip, With<Room>>();
        let world = app.world_mut();
        for mut flip in pans.iter_mut(world) {
            flip.timer = Timer::from_seconds(0.0, TimerMode::Once);
        }
    }

    let world = app.world_mut();
    assert!(
        world.get::<RoomFlip>(room).is_none(),
        "the pan finished but is still on the room, so the arrows stay dead forever",
    );
    assert_eq!(
        world.get::<RoomVariantIndex>(room).map(|index| index.0),
        Some(to),
        "the pan ended on the wrong shot",
    );
    assert_eq!(
        world.get::<RoomTitle>(room).map(|title| title.0),
        Some(def.variants[to].title),
        "the caption still names the shot the player left",
    );
}

/// Every picture the game will show is still held once the game is open.
///
/// The loading overlay waits for the preload list and then despawns, and it used to
/// hold the only handle to every picture in that list. Bevy drops an asset once
/// nothing refers to it, so the whole list went back to being unloaded the moment
/// the overlay disappeared.
///
/// Every room got away with that for as long as there was nothing to see, because a
/// room change happens behind a fade and a fade covers a picture still on its way.
/// The carousel pan has no fade over it: the arrow press swaps the shot for a frame
/// that is not loaded, and what the player sees is the backdrop. So the handles have
/// to outlive the overlay, and the only place that can be pinned is the resource.
///
/// Asserted on the real `spawn_game_ui`, because the claim is about what the running
/// game holds rather than about what the table contains - `pan_frames_are_preloaded`
/// already covers the table's half.
#[test]
fn the_preload_list_is_still_held_once_the_game_is_open() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .init_resource::<PreloadedImages>()
        .init_resource::<RoomFade>()
        .insert_resource(UiScale(1.0))
        .insert_resource(CurrentAct(ActId::ActOne))
        .insert_resource(StartRoom(None))
        // One update, no once-only condition: this is the only frame there is.
        .add_systems(Update, spawn_game_ui);
    app.update();

    let world = app.world();
    let server = world.resource::<AssetServer>();
    let held: Vec<String> = world
        .resource::<PreloadedImages>()
        .handles()
        .iter()
        .filter_map(|handle| server.get_path(handle.id()).map(|path| path.to_string()))
        .collect();

    assert_eq!(
        held.len(),
        all_paths().count(),
        "the game holds {} pictures and the table names {}, so the ones it does not \
         hold will have to load when they are first shown",
        held.len(),
        all_paths().count(),
    );

    for room in [p::F1_HALL, p::F1_HALL_DEAD] {
        for frame in room_def(room, ActId::ActOne).flip.unwrap() {
            assert!(
                held.iter().any(|path| path == frame),
                "{frame} is in the table but nothing holds it, so the pan draws the \
                 backdrop instead of the corridor",
            );
        }
    }
}

/// The control carries a picture of the shot it would lead to, and swaps it when the
/// shot changes.
///
/// Read off the real table and through the real system, because the claim is about
/// what is on the button rather than about what the table says: the table could name
/// both pictures perfectly while the system left the first one up forever.
#[test]
fn the_control_shows_the_shot_it_leads_to() {
    for (from, shown) in [
        (0, p::CAROUSEL_TO_STAIRS),
        (1, p::CAROUSEL_TO_ELEVATOR),
    ] {
        let def = room_def(p::F1_HALL, ActId::ActOne);
        let target = control_target(from, def.variants.len()).expect("two shots");
        assert_eq!(
            def.variants[target].preview,
            Some(shown),
            "standing on shot {from} the control should offer shot {target}",
        );
    }
}

/// A single-picture room shows no control at all.
///
/// The control is one rectangle at the bottom of the screen; a room that cannot be
/// flipped would have it sitting there looking like something to press, and the
/// player would press it and nothing would happen.
#[test]
fn a_room_with_one_shot_hides_the_control() {
    let def = room_def(p::F1_STREET_1, ActId::ActOne);

    assert_eq!(control_target(0, def.variants.len()), None);
    assert!(
        def.variants.iter().all(|variant| variant.preview.is_none()),
        "a single-picture room carries a control picture",
    );
}

/// Pressing the control moves the shot, plays the pan, and leaves the control showing
/// where it would go next.
///
/// The whole path in one test, because the three halves are what make it work: the
/// press has to start the pan rather than cutting, the pan has to land on the shot,
/// and the picture on the button has to follow. A test that only checked the table
/// would pass with the system never writing the picture at all.
#[test]
fn pressing_the_control_moves_shot_picture_and_control_together() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .insert_resource(CurrentAct(ActId::ActOne))
        // `for_each_click` scales the button visuals by it, like every other caller.
        .insert_resource(UiScale(1.0));

    let def = room_def(p::F1_HALL, ActId::ActOne);
    let frames = pan_of(def).len();
    // One chain, as the game registers them. The two systems both write the room's
    // shot and so cannot run unordered against each other, and Bevy checks that when
    // they run rather than at build time.
    app.add_systems(
        Update,
        (
            move |mut commands: Commands,
                  asset_server: Res<AssetServer>,
                  mut once: Local<bool>| {
                if *once {
                    return;
                }
                *once = true;
                crate::scenes::game::rooms::spawn::spawn_room(
                    &mut commands,
                    &asset_server,
                    def,
                    Vec3::ZERO,
                );
            },
            carousel_system,
            room_flip_system,
            carousel_control_system,
        )
            .chain(),
    );
    app.update();

    // The control the way `spawn_game_ui` builds it: one button, one picture.
    let room = app
        .world_mut()
        .query_filtered::<Entity, With<Room>>()
        .iter(app.world())
        .next()
        .expect("the hall spawned");
    let opening = app
        .world()
        .resource::<AssetServer>()
        .load::<Image>(p::CAROUSEL_TO_STAIRS);
    let control = app
        .world_mut()
        .spawn((
            Button,
            CarouselArrow,
            crate::buttons::ButtonSizes::default(),
            ImageNode {
                image: opening,
                ..default()
            },
            Interaction::default(),
            Node::default(),
        ))
        .id();

    app.update();
    let picture_of = |app: &mut App| -> String {
        let world = app.world_mut();
        world
            .resource::<AssetServer>()
            .get_path(world.get::<ImageNode>(control).expect("the control").image.id())
            .map(|path| path.to_string())
            .unwrap_or_default()
    };
    assert_eq!(
        picture_of(&mut app),
        p::CAROUSEL_TO_STAIRS,
        "the control opens on the stairs while the player is at the lift",
    );

    // A press is `Pressed` then `None` over the same button; that pair is what
    // `for_each_click` reads as a completed click.
    *app.world_mut().get_mut::<Interaction>(control).unwrap() = Interaction::Pressed;
    app.update();
    *app.world_mut().get_mut::<Interaction>(control).unwrap() = Interaction::None;
    app.update();

    let world = app.world_mut();
    assert!(
        world.get::<RoomFlip>(room).is_some(),
        "the press cut to the stairs instead of playing the pan",
    );
    assert_eq!(
        world.get::<RoomVariantIndex>(room).map(|index| index.0),
        Some(0),
        "the shot changed while the pan was still playing, so the caption and the \\
         control would name a corridor the player has not arrived at",
    );

    // Run the pan out, then the control should be offering the way back.
    for _ in 0..frames {
        app.update();
        let mut pans = app
            .world_mut()
            .query_filtered::<&mut RoomFlip, With<Room>>();
        let world = app.world_mut();
        for mut flip in pans.iter_mut(world) {
            flip.timer = Timer::from_seconds(0.0, TimerMode::Once);
        }
    }
    app.update();

    assert_eq!(
        app.world().get::<RoomVariantIndex>(room).map(|index| index.0),
        Some(1),
        "the pan landed on the wrong shot",
    );
    assert_eq!(
        picture_of(&mut app),
        p::CAROUSEL_TO_ELEVATOR,
        "the control still offers the stairs while the player is standing on them",
    );
}

/// The lift really does run through its frames while it stands.
///
/// The unit tests above walk `RoomAnim` by hand, which says the bookkeeping is
/// right but says nothing about the room: that `spawn_room` puts the component on
/// the lift at all, that the system is in the schedule, and that it writes the
/// handle onto the sprite that is actually on screen. All three could be wrong
/// with every other test here still green.
#[test]
fn the_lift_plays_its_frames_while_it_stands() {
    use crate::scenes::game::rooms::components::RoomPart;
    use crate::scenes::game::rooms::spawn::spawn_room;
    use crate::scenes::game::systems;

    let def = room_def(p::ELEVATOR, ActId::ActOne);
    let frames = def.anim.expect("the lift plays an animation").to_vec();

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Image>();
    app.add_systems(Update, systems::room_anim_system);
    let world = app.world_mut();
    let asset_server = world.resource::<AssetServer>().clone();
    spawn_room(&mut world.commands(), &asset_server, def, Vec3::ZERO);
    app.update();

    let room = {
        let world = app.world_mut();
        world
            .query_filtered::<Entity, With<Room>>()
            .iter(world)
            .next()
            .expect("the lift spawned")
    };
    assert!(
        app.world().get::<RoomAnim>(room).is_some(),
        "the lift spawned without the animation on it, so it stands on one picture",
    );

    // The frame the room is actually drawing, read off the sprite rather than off
    // the component: the component is what the system reads, and the sprite is
    // what the player sees.
    let showing = |app: &mut App| -> String {
        let world = app.world_mut();
        let part = world
            .query_filtered::<Entity, With<RoomPart>>()
            .iter(world)
            .next()
            .expect("the room picture");
        world
            .resource::<AssetServer>()
            .get_path(world.get::<Sprite>(part).expect("a sprite").image.id())
            .map_or(String::new(), |path| path.to_string())
    };

    assert_eq!(
        showing(&mut app),
        frames[0],
        "the lift does not open on the frame it is supposed to",
    );

    // Wind the timer forward rather than sleeping: `ANIM_FRAME_SECONDS` is a real
    // duration and the test must not take five seconds to say what `advance`
    // already says in the unit test above.
    let to_next_frame = |app: &mut App| {
        let world = app.world_mut();
        if let Some(mut anim) = world.get_mut::<RoomAnim>(room) {
            anim.timer = Timer::from_seconds(0.0, TimerMode::Once);
        }
    };

    for expected in frames.iter().skip(1) {
        to_next_frame(&mut app);
        app.update();
        assert_eq!(
            showing(&mut app),
            *expected,
            "the lift skipped a frame or repeated one, so the fall jumps",
        );
    }

    // Past the last frame the system takes the component off, which is what stops
    // it reading off the end of the array for the rest of the beat.
    to_next_frame(&mut app);
    app.update();
    assert!(
        app.world().get::<RoomAnim>(room).is_none(),
        "the finished animation is still on the room and will keep ticking",
    );
    assert_eq!(
        showing(&mut app),
        frames[frames.len() - 1],
        "a spent animation left the lift on the wrong frame",
    );
}

//! Tests for the room carousel.
//!
//! The modulo here is wrapped arithmetic in both directions, which is the
//! classic place for an off-by-one to hide: a room with one variant must not
//! move, and an empty room must not divide by zero.

use bevy::prelude::*;

use super::systems::{item_hotspot_visibility_system, step};
use crate::acts::Item;
use crate::scenes::game::items::WorldItems;
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

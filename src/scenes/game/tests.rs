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

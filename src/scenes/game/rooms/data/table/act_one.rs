//! Act 1, "The Curse": the walk home, the concierge, and the lift that stops.
//!
//! `F1_CONCIERGE` and `F1_CONCIERGE_DARK` are the same room twice on purpose.
//! Which row answers a lookup depends on the act rather than on where the player
//! is, so the two rows stay adjacent here: reading them next to each other is the
//! only way to see what actually changes between them.

use bevy::prelude::*;

use crate::acts::Item;
// The builders expand into these two, so they have to be nameable here even
// though no row writes them out by hand.
use crate::scenes::game::rooms::components::{HotspotAction, HotspotDef, RoomVariant};
use crate::scenes::sound::{Music, TransitionSound};

// Imported as a glob because the builders expand into each other: `room!` calls
// `shot!`, `apartment!` calls `hop!`, `drop!` and `take!`. Naming only the macros
// an act uses directly is a trap - it compiles until the act uses a builder whose
// own expansion is missing from scope.
use super::super::builders::*;
use super::super::{RoomDef, p};

/// Both halls share this second variant, so it is written once.
///
/// It is a variant rather than a row of its own: both halls offer the stairs, so
/// it would be a second key pointing at one picture and `room_def` would resolve
/// whichever came first.
pub(super) const STAIRS_SHOT: RoomVariant = shot!(
    p::F1_STAIRS,
    "1st floor - stairs",
    "The stairs are open. Climb up.",
    &[hop!(p::STAIRS_1, 200.0, 30.0, Vec2::new(300.0, 450.0))]
);

pub(super) static ROOMS: &[RoomDef] = &[
    // -- Street --------------------------------------------------------------
    room!(
        p::F1_STREET_1,
        "Street in front of home",
        "You are tired after work and going home.\nNow you are approaching the entrance.",
        TransitionSound::NextRoom,
        Music::City,
        &[hop!(p::F1_STREET_2, -190.0, 100.0)]
    ),
    room!(
        p::F1_STREET_2,
        "Street in front of home",
        "Enter the building by clicking on the brown door.",
        TransitionSound::NextRoom,
        Music::City,
        &[hop!(p::F1_CONCIERGE, -10.0, 0.0)]
    ),
    // -- Concierge -----------------------------------------------------------
    // Act 1 only: the pass is spent from the inventory to get through.
    room!(
        p::F1_CONCIERGE,
        "Concierge",
        "Go through the concierge room,\nshowing your pass from the inventory.",
        TransitionSound::NextRoomWithOpenDoor,
        Music::Indoors,
        &[gated!(p::F1_HALL, 0.0, 50.0, Vec2::new(400.0, 400.0), Item::Pass)]
    ),
    // After the basement loop: nobody is on duty, the lights are off, the lift
    // is dead.
    room!(
        p::F1_CONCIERGE_DARK,
        "Concierge",
        "Nobody is at the desk.\nThe concierge waves you through.",
        TransitionSound::NextRoomWithOpenDoor,
        Music::Indoors,
        &[hop!(p::F1_HALL_DEAD, 0.0, 0.0)]
    ),
    // -- Hall ----------------------------------------------------------------
    carousel!(
        TransitionSound::NextRoom,
        Music::Indoors,
        shot!(
            p::F1_HALL,
            "Hall - 1st floor",
            "Choose: take the elevator or\nwalk up the stairs.",
            &[hop!(p::ELEVATOR, 0.0, 25.0, Vec2::new(250.0, 465.0))]
        ),
        STAIRS_SHOT
    ),
    carousel!(
        TransitionSound::NextRoom,
        Music::Indoors,
        shot!(
            p::F1_HALL_DEAD,
            "Hall - 1st floor",
            "The elevator is out of order.\nThe stairs are the only way up.",
            &[]
        ),
        STAIRS_SHOT
    ),
    // -- The lift ------------------------------------------------------------
    // Act 1's last row. The fall is what the player is riding towards, and
    // `B_HALL` in act 2 is where they land - that room, not this one, carries the
    // act change.
    beat!(
        p::ELEVATOR,
        "Inside elevator",
        "You are inside the elevator.\nAfter 15 seconds of riding, you fall and end up in the basement.",
        TransitionSound::ElevatorFall,
        Music::Indoors,
        Some((p::B_HALL, 4.0))
    ),
];

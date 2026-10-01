//! Act 2, "The Descent": the basement, and the long way back up.
//!
//! The stairs live here rather than with floor 2. `STAIRS_1` is reached from the
//! basement corridor, so it belongs to the act the player is climbing out of; it
//! is also where the floor 2 black door sends them when they open it with all
//! three tools. Grouping by act keeps the return path and the way back in the
//! same file, which is the thing that has to stay consistent.

use bevy::prelude::*;

use crate::acts::{ActId, Item};
// Nameable because the builders expand into them; see `act_one`.
use crate::scenes::game::rooms::components::{HotspotAction, HotspotDef, RoomVariant};
use crate::scenes::sound::{Music, TransitionSound};

// A glob, not a named list: see the note in `act_one`. `chapter!` and `room!`
// expand into `shot!`, which is not written here by name.
use super::super::builders::*;
use super::super::{RoomDef, p};

pub(super) static ROOMS: &[RoomDef] = &[
    // This room carries `next_act`, so it is where act 2 begins: the player is
    // standing in act 1 the moment they enter the lift and act 2 the moment they
    // arrive here.
    chapter!(
        p::B_HALL,
        "Basement - elevator hall",
        "You are in the basement.\nThis is where the first act comes to an end.",
        Music::Basement,
        Some((p::B_CORRIDOR, 2.0)),
        ActId::ActTwo
    ),
    // -- Corridor ------------------------------------------------------------
    room!(
        p::B_CORRIDOR,
        "Basement Corridor",
        "",
        TransitionSound::NextRoom,
        Music::Basement,
        &[
            hop!(p::STAIRS_1, 250.0, 0.0),
            hop!(p::B_DEEP, 0.0, 0.0),
            hop!(p::B_EXIT, -320.0, 0.0)
        ]
    ),
    room!(
        p::B_DEEP,
        "Basement Deep",
        "Something is lying on the floor.",
        TransitionSound::NextRoom,
        Music::Basement,
        &[
            hop!(p::B_CORRIDOR, 0.0, 0.0),
            // The teddy, on the floor by the doorway.
            take!(Item::Teddy, 60.0, -210.0)
        ]
    ),
    // -- Back to the street --------------------------------------------------
    room!(
        p::B_EXIT,
        "Basement Exit",
        "The door at the end of the corridor leads up.",
        TransitionSound::NextRoom,
        Music::Basement,
        &[hop!(p::B_STREET_1, 0.0, 0.0)]
    ),
    room!(
        p::B_STREET_1,
        "Stairs to the street",
        "The door at the top is open.",
        TransitionSound::NextRoomWithOpenDoor,
        Music::Basement,
        &[hop!(p::B_STREET_2, 0.0, 200.0)]
    ),
    room!(
        p::B_STREET_2,
        "Courtyard",
        "You are outside. Follow the path to the street.",
        TransitionSound::NextRoom,
        Music::City,
        &[hop!(p::F1_STREET_1, 0.0, -20.0)]
    ),
    // -- The stairs ----------------------------------------------------------
    // Not act 3 material despite being the way up there: the player reaches them
    // from the basement, and floor 2 is only "their floor" once the elevator is
    // behind them. The last row hands off to act 3.
    room!(
        p::STAIRS_1,
        "Stairs",
        "",
        TransitionSound::NextRoom,
        Music::Indoors,
        &[hop!(p::STAIRS_2, 50.0, 0.0)]
    ),
    room!(
        p::STAIRS_2,
        "Stairs",
        "",
        TransitionSound::NextRoom,
        Music::Indoors,
        &[hop!(p::F2_HALL, 50.0, 0.0)]
    ),
];

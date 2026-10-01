//! The room table itself.
//!
//! One row per room. `room_def` looks rows up by `variants[0].path`; that first
//! variant path is the room's key.

use bevy::prelude::*;

use crate::acts::{ActId, Item};
use crate::scenes::game::rooms::components::{HotspotAction, HotspotDef, RoomVariant};
use crate::scenes::sound::{Music, TransitionSound};

use super::builders::{
    HOTSPOT_SIZE, ITEM_HOTSPOT_SIZE, apartment, beat, carousel, chapter, drop, gated, hop, locked,
    room, shot, take,
};
use super::{RoomDef, p};

/// Both halls share this second variant, so it is written once.
const STAIRS_SHOT: RoomVariant = shot!(
    p::F1_STAIRS,
    "1st floor - stairs",
    "The stairs are open. Climb up.",
    &[hop!(p::STAIRS_1, 250.0, 0.0)]
);

pub(super) static ROOMS: &[RoomDef] = &[
    // -- Floor 1 ------------------------------------------------------------
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
    // Act 1 only: the pass is spent from the inventory to get through.
    room!(
        p::F1_CONCIERGE,
        "Concierge",
        "Go through the concierge room,\nshowing your pass from the inventory.",
        TransitionSound::NextRoomWithOpenDoor,
        Music::Indoors,
        &[gated!(p::F1_HALL, 0.0, 0.0, Item::Pass)]
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
    carousel!(
        TransitionSound::NextRoom,
        Music::Indoors,
        shot!(
            p::F1_HALL,
            "Hall - 1st floor",
            "Choose: take the elevator or\nwalk up the stairs.",
            &[hop!(p::ELEVATOR, 0.0, 0.0)]
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
    // -- Basement -----------------------------------------------------------
    beat!(
        p::ELEVATOR,
        "Inside elevator",
        "You are inside the elevator.\nAfter 15 seconds of riding, you fall and end up in the basement.",
        TransitionSound::ElevatorFall,
        Music::Indoors,
        Some((p::B_HALL, 4.0))
    ),
    chapter!(
        p::B_HALL,
        "Basement - elevator hall",
        "You are in the basement.\nThis is where the first act comes to an end.",
        Music::Basement,
        Some((p::B_CORRIDOR, 2.0)),
        ActId::ActTwo
    ),
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
    // -- Stairs -------------------------------------------------------------
    room!(
        p::STAIRS_1,
        "Stairs",
        "",
        TransitionSound::NextRoom,
        Music::Indoors,
        &[hop!(p::STAIRS_2, 0.0, 0.0)]
    ),
    room!(
        p::STAIRS_2,
        "Stairs",
        "",
        TransitionSound::NextRoom,
        Music::Indoors,
        &[hop!(p::F2_HALL, 0.0, 0.0)]
    ),
    // -- Floor 2 ------------------------------------------------------------
    RoomDef {
        sound: TransitionSound::NextRoom,
        music: Music::Indoors,
        interactive: true,
        auto_next: None,
        variants: &[shot!(
            p::F2_HALL,
            "Floor 2",
            "",
            &[
                hop!(p::F2_CORRIDOR, 0.0, 0.0),
                hop!(p::AP_1, -250.0, 0.0),
                hop!(p::AP_2, 250.0, 0.0)
            ]
        )],
        next_act: Some(ActId::ActThree),
    },
    // The corridor: the way back to the hall is dead centre, the left-hand door
    // opens into Apartment 3, and the upper panel of the black door is the one
    // that needs all three tools.
    room!(
        p::F2_CORRIDOR,
        "Floor 2 - Corridor",
        "",
        TransitionSound::NextRoom,
        Music::Indoors,
        &[
            hop!(p::F2_HALL, 0.0, -270.0, Vec2::new(200.0, 100.0)),
            hop!(p::AP_3, -500.0, 0.0, Vec2::new(200.0, 500.0)),
            locked!(
                p::STAIRS_1,
                0.0,
                0.0,
                Vec2::new(400.0, 400.0),
                &Item::DOOR_TOOLS
            )
        ]
    ),
    // The three apartments. Each has a tool on the floor and can hold the teddy:
    // a spot to leave it and a spot to pick it back up, so putting it down is
    // not one-way.
    apartment!(
        p::AP_1,
        "Apartment 1",
        p::F2_HALL,
        0.0,
        0.0,
        -240.0,
        -170.0,
        Item::Crowbar,
        200.0,
        -200.0
    ),
    apartment!(
        p::AP_2,
        "Apartment 2",
        p::F2_HALL,
        0.0,
        0.0,
        210.0,
        -180.0,
        Item::MetalCutters,
        -230.0,
        -190.0
    ),
    // Apartment 3, reached from the corridor. The exit is on the right edge, and
    // its key is the last of the three the black door wants.
    apartment!(
        p::AP_3,
        "Apartment 3",
        p::F2_CORRIDOR,
        540.0,
        0.0,
        -190.0,
        -190.0,
        Item::KeyDoor2,
        250.0,
        -200.0
    ),
    // -- Act 3, not reachable yet -------------------------------------------
    beat!(
        p::MY_FLOOR,
        "My Floor Lobby",
        "You reached your floor.",
        TransitionSound::NextRoom,
        Music::Indoors,
        None
    ),
];

pub(super) static UNKNOWN: RoomDef = beat!(
    "tex/main_fon.png",
    "Unknown room",
    "",
    TransitionSound::None,
    Music::Indoors,
    None
);

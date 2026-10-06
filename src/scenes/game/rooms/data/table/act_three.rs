//! Act 3, "The Escape": floor 2, the three apartments, and the way out.
//!
//! `F2_HALL` carries `next_act: ActThree`, so it opens this act - the player is
//! still in act 2 while they are on the stairs, and in act 3 once they step into
//! the hall. The change is invisible while playing and only matters when a room
//! lookup depends on the act, which today is only the concierge.

use bevy::prelude::*;

use crate::acts::{ActId, Item};
// Nameable because the builders expand into them; see `act_one`.
use crate::scenes::game::rooms::components::{HotspotAction, HotspotDef, RoomVariant};
use crate::scenes::sound::{Music, TransitionSound};

// A glob, not a named list: see the note in `act_one`. `room!` and `apartment!`
// expand into `shot!`, `hop!`, `drop!` and `take!`.
use super::super::builders::*;
use super::super::{RoomDef, p};

pub(super) static ROOMS: &[RoomDef] = &[
    // Act 3's first row. Spelled out rather than written with `room!` because the
    // act change is the only reason it is not an ordinary room.
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
                hop!(p::F2_CORRIDOR, -10.0, 20.0, Vec2::new(100.0, 150.0)),
                hop!(p::AP_1, -285.0, 0.0, Vec2::new(100.0, 400.0)),
                hop!(p::AP_2, 500.0, 0.0, Vec2::new(200.0, 700.0))
            ]
        )],
        next_act: Some(ActId::ActThree),
        flip: None,
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
    //
    // `apartment!` takes its arguments positionally, in the order
    // path, title, exit, exit_x, exit_y, exit_size, teddy_x, teddy_y, tool,
    // tool_x, tool_y. The size sits right after the exit's coordinates, so it is
    // easy to paste it into the teddy's slot instead; that compiles, runs, and
    // moves the teddy rather than resizing the door.
    apartment!(
        p::AP_1,
        "Apartment 1",
        p::F2_HALL,
        -500.0,
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
        500.0,
        0.0,
        210.0,
        -180.0,
        Item::MetalCutters,
        -230.0,
        -190.0
    ),
    // Apartment 3, reached from the corridor. The exit is on the right edge, and
    // its key is the last of the three the black door wants. The only apartment
    // whose exit is not the default size.
    apartment!(
        p::AP_3,
        "Apartment 3",
        p::F2_CORRIDOR,
        555.0,
        5.0,
        Vec2::new(150.0, 610.0),
        -190.0,
        -190.0,
        Item::KeyDoor2,
        250.0,
        -200.0
    ),
    // -- Not reachable yet ---------------------------------------------------
    // The lobby above floor 2. Nothing links here, so it is a dead end the player
    // cannot walk into; `data::tests` keeps it in a `PENDING` list that fails the
    // moment something does link to it.
    beat!(
        p::MY_FLOOR,
        "My Floor Lobby",
        "You reached your floor.",
        TransitionSound::NextRoom,
        Music::Indoors,
        None
    ),
];

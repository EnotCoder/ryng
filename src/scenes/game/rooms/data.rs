//! Room table: one static row per room, looked up by asset path.
//!
//! Adding a room = one row below. Adding an asset = one constant in [`p`].
//! `all_paths` is derived from the table, so the preload list can no longer
//! drift out of sync with the rooms that actually exist.

use bevy::prelude::*;

use crate::acts::{ActId, Item};
use crate::scenes::game::rooms::components::{HotspotAction, HotspotDef, RoomVariant};
use crate::scenes::sound::TransitionSound;

#[derive(Clone, Copy)]
pub(crate) struct RoomDef {
    pub sound: TransitionSound,
    pub interactive: bool,
    pub auto_next: Option<(&'static str, f32)>,
    pub variants: &'static [RoomVariant],
    pub next_act: Option<ActId>,
}

/// Asset paths in one place. `F1_STAIRS` is only ever a *variant* picture (the
/// carousel target of both halls), never a lookup key.
pub(crate) mod p {
    pub const F1_STREET_1: &str = "tex/rooms/floor_1/street_to_home_1.png";
    pub const F1_STREET_2: &str = "tex/rooms/floor_1/street_to_home_2.png";
    pub const F1_CONCIERGE: &str = "tex/rooms/floor_1/room_concierge.png";
    pub const F1_CONCIERGE_DARK: &str = "tex/rooms/floor_1/room_concierge_dark.png";
    pub const F1_HALL: &str = "tex/rooms/floor_1/room_with_elevator_floor_1.png";
    pub const F1_HALL_DEAD: &str = "tex/rooms/floor_1/room_with_elevator_floor_1_dont_work.png";
    pub const F1_STAIRS: &str = "tex/rooms/floor_1/stairs_1_floor.png";
    pub const ELEVATOR: &str = "tex/rooms/elevator_Inside.png";
    pub const B_HALL: &str = "tex/rooms/basement/basement_with_elevator.png";
    pub const B_CORRIDOR: &str = "tex/rooms/basement/basement_stairs.png";
    pub const B_DEEP: &str = "tex/rooms/basement/basement_stairs_center_room.png";
    pub const B_EXIT: &str = "tex/rooms/basement/basement_stairs_left_room.png";
    pub const B_STREET_1: &str = "tex/rooms/basement/stairs_to_street_1.png";
    pub const B_STREET_2: &str = "tex/rooms/basement/stairs_to_street_2.png";
    pub const STAIRS_1: &str = "tex/rooms/stairs/stairs_1.png";
    pub const STAIRS_2: &str = "tex/rooms/stairs/stairs_2.png";
    pub const F2_HALL: &str = "tex/rooms/floor_2/room_1.png";
    pub const F2_CORRIDOR: &str = "tex/rooms/floor_2/room_2.png";
    pub const AP_1: &str = "tex/rooms/floor_2/ap_1.png";
    pub const AP_2: &str = "tex/rooms/floor_2/ap_2.png";
    pub const MY_FLOOR: &str = "tex/rooms/my_floor/room_with_elevator_floor_my.png";
}

// --------------------------------------------------------------- builders
//
// These are macros rather than `const fn`s on purpose. A `&[..]` can only be
// promoted to `'static` when it is written syntactically where the borrow
// happens: a `const fn` that fills the array from its own parameters would
// produce a temporary that dies with the frame (E0716). Expanding the macro at
// the `static` puts every literal straight into the constant initializer.

const HOTSPOT_SIZE: Vec2 = Vec2::new(200.0, 300.0);

/// A hotspot leading to `target`.
macro_rules! hop {
    ($target:expr, $x:expr, $y:expr) => {
        HotspotDef {
            action: HotspotAction::GoToRoom($target),
            pos: Vec2::new($x, $y),
            size: HOTSPOT_SIZE,
            gate: None,
        }
    };
}

/// A hotspot that only opens if the player holds `$item` in the active slot.
macro_rules! gated {
    ($target:expr, $x:expr, $y:expr, $item:expr) => {
        HotspotDef {
            action: HotspotAction::GoToRoom($target),
            pos: Vec2::new($x, $y),
            size: HOTSPOT_SIZE,
            gate: Some($item),
        }
    };
}

macro_rules! shot {
    ($path:expr, $title:expr, $story:expr, $hotspots:expr) => {
        RoomVariant {
            path: $path,
            title: $title,
            story: $story,
            hotspots: $hotspots,
        }
    };
}

/// One picture, one variant, player-driven. Covers most of the rooms.
macro_rules! room {
    ($path:expr, $title:expr, $story:expr, $sound:expr, $hotspots:expr) => {
        RoomDef {
            sound: $sound,
            interactive: true,
            auto_next: None,
            variants: &[shot!($path, $title, $story, $hotspots)],
            next_act: None,
        }
    };
}

/// A non-interactive beat: it plays, waits, then moves on by itself.
macro_rules! beat {
    ($path:expr, $title:expr, $story:expr, $sound:expr, $auto_next:expr) => {
        RoomDef {
            sound: $sound,
            interactive: false,
            auto_next: $auto_next,
            variants: &[shot!($path, $title, $story, &[])],
            next_act: None,
        }
    };
}

/// A beat that also advances the act counter.
macro_rules! chapter {
    ($path:expr, $title:expr, $story:expr, $auto_next:expr, $act:expr) => {
        RoomDef {
            sound: TransitionSound::None,
            interactive: false,
            auto_next: $auto_next,
            variants: &[shot!($path, $title, $story, &[])],
            next_act: Some($act),
        }
    };
}

/// Two or more pictures the player flips between.
macro_rules! carousel {
    ($sound:expr, $($variant:expr),+ $(,)?) => {
        RoomDef {
            sound: $sound,
            interactive: true,
            auto_next: None,
            variants: &[$($variant),+],
            next_act: None,
        }
    };
}

/// Floor 2 side rooms: one door back to the hall, no story.
macro_rules! side_room {
    ($path:expr, $title:expr) => {
        room!(
            $path,
            $title,
            "",
            TransitionSound::NextRoom,
            &[hop!(p::F2_HALL, 0.0, 0.0)]
        )
    };
}

/// Both halls share this second variant, so it is written once.
const STAIRS_SHOT: RoomVariant = shot!(
    p::F1_STAIRS,
    "1st floor - stairs",
    "The stairs are open. Climb up.",
    &[hop!(p::STAIRS_1, 250.0, 0.0)]
);

// ------------------------------------------------------------------ table

static ROOMS: &[RoomDef] = &[
    // -- Floor 1 ------------------------------------------------------------
    room!(
        p::F1_STREET_1,
        "Street in front of home",
        "You are tired after work and going home.\nNow you are approaching the entrance.",
        TransitionSound::NextRoom,
        &[hop!(p::F1_STREET_2, -190.0, 100.0)]
    ),
    room!(
        p::F1_STREET_2,
        "Street in front of home",
        "Enter the building by clicking on the brown door.",
        TransitionSound::NextRoom,
        &[hop!(p::F1_CONCIERGE, -10.0, 0.0)]
    ),
    // Act 1 only: the pass is spent from the inventory to get through.
    room!(
        p::F1_CONCIERGE,
        "Concierge",
        "Go through the concierge room,\nshowing your pass from the inventory.",
        TransitionSound::NextRoomWithOpenDoor,
        &[gated!(p::F1_HALL, 0.0, 0.0, Item::Pass)]
    ),
    // After the basement loop: nobody is on duty, the lights are off, the lift
    // is dead.
    room!(
        p::F1_CONCIERGE_DARK,
        "Concierge",
        "Nobody is at the desk.\nThe concierge waves you through.",
        TransitionSound::NextRoomWithOpenDoor,
        &[hop!(p::F1_HALL_DEAD, 0.0, 0.0)]
    ),
    carousel!(
        TransitionSound::NextRoom,
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
        Some((p::B_HALL, 4.0))
    ),
    chapter!(
        p::B_HALL,
        "Basement - elevator hall",
        "You are in the basement.\nThis is where the first act comes to an end.",
        Some((p::B_CORRIDOR, 2.0)),
        ActId::ActTwo
    ),
    room!(
        p::B_CORRIDOR,
        "Basement Corridor",
        "",
        TransitionSound::NextRoom,
        &[
            hop!(p::STAIRS_1, 250.0, 0.0),
            hop!(p::B_DEEP, 0.0, 0.0),
            hop!(p::B_EXIT, -320.0, 0.0)
        ]
    ),
    room!(
        p::B_DEEP,
        "Basement Deep",
        "",
        TransitionSound::NextRoom,
        &[hop!(p::B_CORRIDOR, 0.0, 0.0)]
    ),
    room!(
        p::B_EXIT,
        "Basement Exit",
        "The door at the end of the corridor leads up.",
        TransitionSound::NextRoom,
        &[hop!(p::B_STREET_1, 0.0, 0.0)]
    ),
    room!(
        p::B_STREET_1,
        "Stairs to the street",
        "The door at the top is open.",
        TransitionSound::NextRoomWithOpenDoor,
        &[hop!(p::B_STREET_2, 0.0, 200.0)]
    ),
    room!(
        p::B_STREET_2,
        "Courtyard",
        "You are outside. Follow the path to the street.",
        TransitionSound::NextRoom,
        &[hop!(p::F1_STREET_1, 0.0, -20.0)]
    ),
    // -- Stairs -------------------------------------------------------------
    room!(
        p::STAIRS_1,
        "Stairs",
        "",
        TransitionSound::NextRoom,
        &[hop!(p::STAIRS_2, 0.0, 0.0)]
    ),
    room!(
        p::STAIRS_2,
        "Stairs",
        "",
        TransitionSound::NextRoom,
        &[hop!(p::F2_HALL, 0.0, 0.0)]
    ),
    // -- Floor 2 ------------------------------------------------------------
    RoomDef {
        sound: TransitionSound::NextRoom,
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
    side_room!(p::F2_CORRIDOR, "Floor 2 - Corridor"),
    side_room!(p::AP_1, "Apartment 1"),
    side_room!(p::AP_2, "Apartment 2"),
    // -- Act 3, not reachable yet -------------------------------------------
    beat!(
        p::MY_FLOOR,
        "My Floor Lobby",
        "You reached your floor.",
        TransitionSound::NextRoom,
        None
    ),
];

static UNKNOWN: RoomDef = beat!(
    "tex/main_fon.png",
    "Unknown room",
    "",
    TransitionSound::None,
    None
);

// ----------------------------------------------------------------- lookup

/// `act` is the act the player is currently in: the concierge asks for the pass
/// and the elevator still works during act 1, after the basement loop it does
/// not.
pub(crate) fn room_def(path: &'static str, act: ActId) -> RoomDef {
    let key = if act != ActId::ActOne && path == p::F1_CONCIERGE {
        p::F1_CONCIERGE_DARK
    } else {
        path
    };
    ROOMS
        .iter()
        .find(|room| room.variants[0].path == key)
        .copied()
        .unwrap_or(UNKNOWN)
}

/// Every picture the game needs, for the loading overlay.
pub(crate) fn all_paths() -> impl Iterator<Item = &'static str> {
    ROOMS
        .iter()
        .flat_map(|room| room.variants.iter())
        .map(|variant| variant.path)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn is_room(path: &str) -> bool {
        ROOMS.iter().any(|room| room.variants[0].path == path)
    }

    /// Nothing may link to a room that is not in the table: `room_def` would
    /// silently hand back `UNKNOWN` and the player would land on a black screen.
    #[test]
    fn every_link_resolves() {
        for room in ROOMS {
            let links = room
                .variants
                .iter()
                .flat_map(|variant| variant.hotspots.iter())
                .map(|hotspot| match hotspot.action {
                    HotspotAction::GoToRoom(target) => target,
                })
                .chain(room.auto_next.map(|(target, _)| target));
            for target in links {
                assert!(
                    is_room(target),
                    "{} -> {target} is not a key in ROOMS",
                    room.variants[0].path,
                );
            }
        }
    }

    /// The lookup takes the first match, so a duplicated key makes a dead row.
    #[test]
    fn keys_are_unique() {
        let keys: HashSet<_> = ROOMS.iter().map(|room| room.variants[0].path).collect();
        assert_eq!(keys.len(), ROOMS.len(), "two rows share a key");
    }

    /// The concierge is the only act-dependent room, and it is easy to invert.
    #[test]
    fn concierge_is_dark_after_act_one() {
        let lit = room_def(p::F1_CONCIERGE, ActId::ActOne);
        assert_eq!(lit.variants[0].path, p::F1_CONCIERGE);
        for act in [ActId::ActTwo, ActId::ActThree] {
            let dark = room_def(p::F1_CONCIERGE, act);
            assert_eq!(dark.variants[0].path, p::F1_CONCIERGE_DARK);
        }
    }
}

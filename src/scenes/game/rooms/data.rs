use bevy::prelude::*;

use crate::acts::{ActId, Item};
use crate::scenes::game::rooms::components::{HotspotAction, HotspotDef, RoomVariant};
use crate::scenes::sound::TransitionSound;

pub(crate) struct RoomDef {
    pub sound: TransitionSound,
    pub interactive: bool,
    pub auto_next: Option<(&'static str, f32)>,
    pub variants: Vec<RoomVariant>,
    pub next_act: Option<ActId>,
}

fn room_variant(
    path: &'static str,
    title: &'static str,
    story: &'static str,
    hotspots: Vec<HotspotDef>,
) -> RoomVariant {
    RoomVariant {
        path,
        title,
        story,
        hotspots,
    }
}

fn go_hotspot(target: &'static str, pos: Vec2) -> HotspotDef {
    HotspotDef {
        action: HotspotAction::GoToRoom(target),
        pos,
        size: Vec2::new(200.0, 300.0),
        gate: None,
    }
}

fn gated_hotspot(target: &'static str, pos: Vec2, gate: Item) -> HotspotDef {
    let mut hotspot = go_hotspot(target, pos);
    hotspot.gate = Some(gate);
    hotspot
}

fn simple_room(
    path: &'static str,
    title: &'static str,
    story: &'static str,
    sound: TransitionSound,
    hotspots: Vec<HotspotDef>,
) -> RoomDef {
    RoomDef {
        sound,
        interactive: true,
        auto_next: None,
        variants: vec![room_variant(path, title, story, hotspots)],
        next_act: None,
    }
}

fn story_room(
    path: &'static str,
    title: &'static str,
    story: &'static str,
    sound: TransitionSound,
    auto_next: Option<(&'static str, f32)>,
) -> RoomDef {
    RoomDef {
        sound,
        interactive: false,
        auto_next,
        variants: vec![room_variant(path, title, story, Vec::new())],
        next_act: None,
    }
}

pub(crate) fn room_def(path: &'static str) -> RoomDef {
    match path {
        "tex/rooms/floor_1/street_to_home_1.png" => {
            let next = "tex/rooms/floor_1/street_to_home_2.png";
            simple_room(
                "tex/rooms/floor_1/street_to_home_1.png",
                "Street in front of home",
                "You are tired after work and going home.\nNow you are approaching the entrance.",
                TransitionSound::NextRoom,
                vec![go_hotspot(next, Vec2::new(-190.0, 100.0))],
            )
        }
        "tex/rooms/floor_1/street_to_home_2.png" => {
            let next = "tex/rooms/floor_1/room_concierge.png";
            simple_room(
                "tex/rooms/floor_1/street_to_home_2.png",
                "Street in front of home",
                "Enter the building by clicking on the brown door.",
                TransitionSound::NextRoom,
                vec![go_hotspot(next, Vec2::new(-10.0, 0.0))],
            )
        }
        "tex/rooms/floor_1/room_concierge.png" => {
            let next = "tex/rooms/floor_1/room_with_elevator_floor_1.png";
            simple_room(
                "tex/rooms/floor_1/room_concierge.png",
                "Concierge",
                "Go through the concierge room,\nshowing your pass from the inventory.",
                TransitionSound::NextRoomWithOpenDoor,
                vec![gated_hotspot(next, Vec2::new(0.0, 0.0), Item::Pass)],
            )
        }
        "tex/rooms/floor_1/room_with_elevator_floor_1.png"
        | "tex/rooms/floor_1/stairs_1_floor.png" => RoomDef {
            sound: TransitionSound::NextRoom,
            interactive: true,
            auto_next: None,
            variants: vec![
                room_variant(
                    "tex/rooms/floor_1/room_with_elevator_floor_1.png",
                    "Hall - 1st floor",
                    "Choose: take the elevator or\nwalk up the stairs.",
                    vec![go_hotspot(
                        "tex/rooms/elevator_Inside.png",
                        Vec2::new(0.0, 0.0),
                    )],
                ),
                room_variant(
                    "tex/rooms/floor_1/stairs_1_floor.png",
                    "1st floor - stairs",
                    "Taking the stairs will start you from level 2.\n(The stairs are closed for now - wait for the next acts)",
                    Vec::new(),
                ),
            ],
            next_act: None,
        },
        "tex/rooms/elevator_Inside.png" => story_room(
            "tex/rooms/elevator_Inside.png",
            "Inside elevator",
            "You are inside the elevator.\nAfter 15 seconds of riding, you fall and end up in the basement.",
            TransitionSound::ElevatorFall,
            Some(("tex/rooms/basement/basement_with_elevator.png", 4.0)),
        ),
        "tex/rooms/basement/basement_with_elevator.png" => RoomDef {
            sound: TransitionSound::None,
            interactive: false,
            auto_next: Some(("tex/rooms/basement/basement_stairs_left_room.png", 2.0)),
            variants: vec![room_variant(
                "tex/rooms/basement/basement_with_elevator.png",
                "Basement - elevator hall",
                "You are in the basement.\nThis is where the first act comes to an end.",
                Vec::new(),
            )],
            next_act: Some(ActId::ActTwo),
        },
        "tex/rooms/basement/basement_stairs_left_room.png" => {
            let next = "tex/rooms/basement/basement_stairs.png";
            simple_room(
                "tex/rooms/basement/basement_stairs_left_room.png",
                "Basement Entrance",
                "",
                TransitionSound::NextRoom,
                vec![go_hotspot(next, Vec2::new(0.0, 0.0))],
            )
        }
        "tex/rooms/basement/basement_stairs.png" => {
            let next = "tex/rooms/basement/basement_stairs_center_room.png";
            simple_room(
                "tex/rooms/basement/basement_stairs.png",
                "Basement Corridor",
                "",
                TransitionSound::NextRoom,
                vec![go_hotspot(next, Vec2::new(0.0, 0.0))],
            )
        }
        "tex/rooms/basement/basement_stairs_center_room.png" => RoomDef {
            sound: TransitionSound::None,
            interactive: false,
            auto_next: Some(("tex/rooms/my_floor/room_with_elevator_floor_my.png", 2.0)),
            variants: vec![room_variant(
                "tex/rooms/basement/basement_stairs_center_room.png",
                "Basement Deep",
                "",
                Vec::new(),
            )],
            next_act: Some(ActId::ActThree),
        },
        "tex/rooms/my_floor/room_with_elevator_floor_my.png" => story_room(
            "tex/rooms/my_floor/room_with_elevator_floor_my.png",
            "My Floor Lobby",
            "You reached your floor.",
            TransitionSound::NextRoom,
            None,
        ),
        _ => story_room(
            "tex/rooms/main_fon.png",
            "Unknown room",
            "",
            TransitionSound::None,
            None,
        ),
    }
}

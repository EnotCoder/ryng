use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Pass,
}

#[derive(Resource)]
pub struct Inventory(pub Vec<Item>);

impl Default for Inventory {
    fn default() -> Self {
        Self(vec![Item::Pass])
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActId {
    ActOne,
    ActTwo,
    ActThree,
}

pub struct Act {
    pub id: ActId,
    pub name: &'static str,
    pub start_room: &'static str,
}

pub fn get_act(id: ActId) -> Act {
    match id {
        ActId::ActOne => Act {
            id: ActId::ActOne,
            name: "The Curse",
            start_room: "tex/rooms/floor_1/street_to_home_1.png",
        },
        ActId::ActTwo => Act {
            id: ActId::ActTwo,
            name: "The Descent",
            start_room: "tex/rooms/basement/basement_with_elevator.png",
        },
        ActId::ActThree => Act {
            id: ActId::ActThree,
            name: "The Escape",
            start_room: "tex/rooms/floor_2/room_1.png",
        },
    }
}

pub fn default_act() -> Act {
    get_act(ActId::ActOne)
}

#[derive(Resource, Default)]
pub struct CurrentAct(pub ActId);

impl Default for ActId {
    fn default() -> Self {
        ActId::ActOne
    }
}

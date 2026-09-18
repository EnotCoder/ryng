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
}

pub struct Act {
    pub id: ActId,
    pub name: &'static str,
    pub start_room: &'static str,
}

pub fn default_act() -> Act {
    Act {
        id: ActId::ActOne,
        name: "The Curse",
        start_room: "tex/rooms/floor_1/street_to_home_1.png",
    }
}
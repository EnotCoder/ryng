use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Item {
    Pass,
    MainKey,
    Teddy,
    Crowbar,
    MetalCutters,
    KeyDoor2,
}

impl Item {
    /// The inventory icon for this item, and the same picture the item is drawn
    /// with while it is lying on the floor. Keeping the path with the variant
    /// means a new item is one line here plus one in the icon table, rather than
    /// a new field on `InventoryTextures` and another match to remember.
    pub fn icon_path(self) -> &'static str {
        match self {
            Item::Pass => "tex/ui/icons_inv/kon_card.png",
            Item::MainKey => "tex/ui/icons_inv/main_key.png",
            Item::Teddy => "tex/ui/icons_inv/teddy.png",
            Item::Crowbar => "tex/ui/icons_inv/crowbar.png",
            Item::MetalCutters => "tex/ui/icons_inv/metal_cutters.png",
            Item::KeyDoor2 => "tex/ui/icons_inv/key_door_2_floor.png",
        }
    }

    /// The three tools that open the black door on floor 2. A door does not ask
    /// for these one at a time, it needs the set, so they are named as a group
    /// rather than repeated at every use site.
    pub const DOOR_TOOLS: [Item; 3] = [Item::Crowbar, Item::MetalCutters, Item::KeyDoor2];
}

/// One entry per inventory slot, `None` = empty slot.
#[derive(Resource, Clone, PartialEq, Eq, Debug)]
pub struct Inventory(pub Vec<Option<Item>>);

impl Default for Inventory {
    fn default() -> Self {
        // Slot 1: concierge pass, slot 4: the apartment key (stays with you the whole game).
        Self(vec![Some(Item::Pass), None, None, Some(Item::MainKey)])
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ActId {
    /// The first act is the default one: the game starts here, so anything that
    /// needs an act before one is set gets the start of the story.
    #[default]
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

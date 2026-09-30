use bevy::prelude::*;

use crate::acts::Item;

#[derive(Component, Clone, Copy)]
pub enum HotspotAction {
    GoToRoom(&'static str),
    /// Pick the item up into the first free inventory slot.
    Take(Item),
    /// Put the item down, if it is held.
    Drop(Item),
}

#[derive(Component)]
pub struct Hotspot;

#[derive(Component)]
pub struct HotspotIcon;

#[derive(Component)]
pub struct Room;

#[derive(Component)]
pub struct RoomTitle(pub &'static str);

#[derive(Component)]
pub struct RoomStory(pub &'static str);

/// Children (sprite + hotspots) that are replaced on variant switch.
#[derive(Component)]
pub struct RoomPart;

#[derive(Component)]
pub struct RoomVariants(pub &'static [RoomVariant]);

#[derive(Component)]
pub struct RoomVariantIndex(pub usize);

#[derive(Clone, Copy)]
pub struct RoomVariant {
    pub path: &'static str,
    pub title: &'static str,
    pub story: &'static str,
    pub hotspots: &'static [HotspotDef],
}

#[derive(Component, Clone, Copy)]
pub struct HotspotDef {
    pub action: HotspotAction,
    pub pos: Vec2,
    pub size: Vec2,
    /// Spend this one item to pass, as the concierge desk does with the pass.
    /// Empty for an ordinary door.
    pub gate: Option<Item>,
    /// Pass only while carrying all of these. Unlike `gate` they are not
    /// consumed - the tools on the floor 2 door stay in the bag afterwards.
    pub require: &'static [Item],
}

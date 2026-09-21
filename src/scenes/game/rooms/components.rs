use bevy::prelude::*;

use crate::acts::Item;

#[derive(Component, Clone)]
pub enum HotspotAction {
    GoToRoom(&'static str),
}

#[derive(Component)]
pub struct Hotspot;

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
pub struct RoomVariants(pub Vec<RoomVariant>);

#[derive(Component)]
pub struct RoomVariantIndex(pub usize);

#[derive(Clone)]
pub struct RoomVariant {
    pub path: &'static str,
    pub title: &'static str,
    pub story: &'static str,
    pub hotspots: Vec<HotspotDef>,
}

#[derive(Clone)]
pub struct HotspotDef {
    pub action: HotspotAction,
    pub pos: Vec2,
    pub size: Vec2,
    pub gate: Option<Item>,
}

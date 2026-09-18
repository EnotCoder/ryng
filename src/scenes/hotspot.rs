use bevy::prelude::*;

use crate::acts::Item;
use crate::state::GameState;

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

pub fn spawn_room(
    commands: &mut Commands,
    asset_server: &AssetServer,
    variants: Vec<RoomVariant>,
    pos: Vec3,
    interactive: bool,
) {
    let first = variants[0].clone();
    let mut root = commands.spawn((
        Room,
        RoomTitle(first.title),
        RoomStory(first.story),
        RoomVariants(variants),
        RoomVariantIndex(0),
        Transform::from_translation(pos),
        DespawnOnExit(GameState::Game),
    ));
    root.with_children(|parent| spawn_room_content(parent, asset_server, &first, interactive));
}

pub fn spawn_room_content(
    parent: &mut ChildSpawnerCommands<'_>,
    asset_server: &AssetServer,
    variant: &RoomVariant,
    interactive: bool,
) {
    parent.spawn((
        RoomPart,
        Sprite::from_image(asset_server.load(variant.path)),
    ));
    if !interactive {
        return;
    }
    let hotspot_color = if crate::DEBUG_SHOW_HOTSPOTS {
        Color::srgba(1.0, 0.0, 0.3, 0.6)
    } else {
        Color::NONE
    };
    for def in &variant.hotspots {
        let mut child = parent.spawn((
            Hotspot,
            RoomPart,
            def.action.clone(),
            Sprite::from_color(hotspot_color, def.size),
            Transform::from_xyz(def.pos.x, def.pos.y, 1.0),
            Pickable::default(),
        ));
        if let Some(gate) = def.gate {
            child.insert(gate);
        }
    }
}
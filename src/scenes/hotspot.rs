use bevy::prelude::*;

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

pub struct HotspotDef {
    pub action: HotspotAction,
    pub pos: Vec2,
    pub size: Vec2,
}

pub fn spawn_room(
    commands: &mut Commands,
    asset_server: &AssetServer,
    room_path: &'static str,
    hotspots: &[HotspotDef],
    pos: Vec3,
    title: &'static str,
) {
    let mut root = commands.spawn((
        Room,
        RoomTitle(title),
        Transform::from_translation(pos),
        DespawnOnExit(GameState::Game),
    ));
    let hotspot_color = if crate::DEBUG_SHOW_HOTSPOTS {
        Color::srgba(1.0, 0.0, 0.3, 0.6)
    } else {
        Color::NONE
    };
    root.with_children(|parent| {
        parent.spawn(Sprite::from_image(asset_server.load(room_path)));
        for def in hotspots {
            parent.spawn((
                Hotspot,
                def.action.clone(),
                Sprite::from_color(hotspot_color, def.size),
                Transform::from_xyz(def.pos.x, def.pos.y, 1.0),
                Pickable::default(),
            ));
        }
    });
}
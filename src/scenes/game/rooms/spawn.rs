use bevy::prelude::*;

use crate::scenes::game::rooms::components::{
    Hotspot, HotspotIcon, Room, RoomPart, RoomStory, RoomTitle, RoomVariant, RoomVariantIndex,
    RoomVariants,
};
use crate::state::GameState;

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
        let side = def.size.x.min(def.size.y) / 20.0;
        parent.spawn((
            RoomPart,
            HotspotIcon,
            Sprite {
                image: asset_server.load("tex/ui/cheak_room.png"),
                custom_size: Some(Vec2::splat(side)),
                ..default()
            },
            Transform::from_xyz(def.pos.x, def.pos.y, 1.0),
            Pickable::IGNORE,
        ));
    }
}

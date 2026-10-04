use bevy::prelude::*;

use crate::scenes::game::items::ItemHotspot;
use crate::scenes::game::rooms::components::{
    HOTSPOT_ICON_SIZE, Hotspot, HotspotAction, HotspotIcon, Room, RoomPart, RoomStory, RoomTitle,
    RoomVariant, RoomVariantIndex, RoomVariants,
};
use crate::scenes::game::rooms::data::RoomDef;
use crate::state::GameState;

/// Fill for every hotspot when `DEBUG_SHOW_HOTSPOTS` is on. Hotspots are
/// otherwise invisible - they are a rectangle of clickable nothing - so this is
/// the only way to see where the table actually put them.
const DEBUG_HOTSPOT_COLOR: Color = Color::srgba(1.0, 0.0, 0.3, 0.6);

/// The room picture sits at z 0 and everything the player can interact with is
/// layered above it: item sprites at 0.5, an NPC's picture at 0.6, the hotspots at
/// 1.0, and an NPC's click target at 1.5.
///
/// The order between the two clickable layers is load-bearing, not cosmetic. The
/// concierge's way out is a hotspot and the granny stands in front of it, so
/// whichever is higher takes the click - and the whole point of her is that she
/// speaks when you click her rather than the player leaving the room.
#[cfg(test)]
pub(crate) fn test_hotspot_z() -> f32 {
    HOTSPOT_Z
}

pub(crate) const HOTSPOT_Z: f32 = 1.0;

pub(crate) fn spawn_room(
    commands: &mut Commands,
    asset_server: &AssetServer,
    def: RoomDef,
    pos: Vec3,
) {
    let first = def.variants[0];
    let mut root = commands.spawn((
        Room,
        def,
        RoomTitle(first.title),
        RoomStory(first.story),
        RoomVariants(def.variants),
        RoomVariantIndex(0),
        Transform::from_translation(pos),
        // Children (sprites) have `InheritedVisibility`; the parent needs it too,
        // otherwise Bevy logs B0004 hierarchy warnings on every room spawn.
        Visibility::default(),
        DespawnOnExit(GameState::Game),
    ));
    root.with_children(|parent| spawn_room_content(parent, asset_server, &first, def.interactive));
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
        DEBUG_HOTSPOT_COLOR
    } else {
        Color::NONE
    };
    for def in variant.hotspots {
        let mut child = parent.spawn((
            Hotspot,
            RoomPart,
            def.action,
            // The whole definition, so a system can read the tool requirement
            // without keeping a parallel copy of it.
            *def,
            Sprite::from_color(hotspot_color, def.size),
            Transform::from_xyz(def.pos.x, def.pos.y, HOTSPOT_Z),
            Pickable::default(),
        ));
        if let Some(gate) = def.gate {
            child.insert(gate);
        }
        // Pickup and put-down spots come and go with the item, so they are
        // marked to be found without walking every door in the room.
        let is_item_hotspot = matches!(def.action, HotspotAction::Take(_) | HotspotAction::Drop(_));
        if is_item_hotspot {
            child.insert(ItemHotspot);
        }
        // A pickup hotspot sits on the item itself, which is the affordance; a
        // blinking arrow on top of the teddy would only get in the way.
        if matches!(def.action, HotspotAction::Take(_)) {
            continue;
        }
        parent.spawn((
            RoomPart,
            HotspotIcon,
            Sprite {
                image: asset_server.load("tex/ui/cheak_room.png"),
                custom_size: Some(HOTSPOT_ICON_SIZE),
                ..default()
            },
            Transform::from_xyz(def.pos.x, def.pos.y, 1.0),
            Pickable::IGNORE,
        ));
    }
}

use bevy::prelude::*;

use bevy::picking::hover::Hovered;

use crate::scenes::game::items::ItemHotspot;
use crate::scenes::game::rooms::components::{
    HOTSPOT_ICON_SIZE, HOTSPOT_OUTLINE_THICKNESS, HOTSPOT_OUTLINE_Z, Hotspot, HotspotAction,
    HotspotIcon, HotspotOutline, OUTLINE_COLOR, Room, RoomPart, RoomStory, RoomTitle,
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
            // bevy_picking maintains this once it is present, so the hover
            // outline needs no `Pointer<Over>` reader of its own.
            Hovered::default(),
        ));
        if let Some(gate) = def.gate {
            child.insert(gate);
        }
        spawn_outline(&mut child, def.size);
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

/// Draws a hotspot's hover outline as four bars around its rectangle.
///
/// Children of the hotspot rather than siblings: the hotspot already carries the
/// position, so the bars are placed relative to its origin and follow it. They are
/// drawn on top of the hotspot's own sprite, which is invisible - that is the
/// point, the outline is the affordance the room does not otherwise have.
///
/// Four bars and not a bordered texture because the hotspot's size comes from the
/// table and is different for every row; anything drawn as a picture would have to
/// be rebuilt per size, and `Sprite::from_color` needs no asset.
///
/// Hidden until something hovers: the system in `systems` shows them, and hiding
/// rather than drawing at alpha 0 means the bars are not in the picking backend at
/// all.
fn spawn_outline(hotspot: &mut EntityCommands, size: Vec2) {
    let t = HOTSPOT_OUTLINE_THICKNESS;
    // The bars are laid out on the hotspot's own edges rather than straddling
    // them, so the outline sits *on* the clickable area and the middle of the
    // rectangle - where the player is aiming - stays clear. A straddling bar
    // would read as a slightly smaller door than the one they are about to click.
    let (half_w, half_h) = (size.x / 2.0, size.y / 2.0);
    // Full length, never clamped to the thickness. An earlier version clamped both
    // with `.min(t * 4.0)`, on the theory that a hotspot narrower than two bars
    // needed a smaller frame - but that capped the *length* of every bar at 20px,
    // so a 200px door got a 20px dash floating in the middle of each edge rather
    // than a frame. The horizontal bars span the width and the vertical bars the
    // height; where the two meet they overlap, and that overlap is the corner.
    let bar_w = size.x + t;
    let bar_h = size.y + t;

    let bars = [
        // top and bottom: full width, one bar tall
        (Vec2::new(0.0, half_h), Vec2::new(bar_w, t)),
        (Vec2::new(0.0, -half_h), Vec2::new(bar_w, t)),
        // left and right: one bar wide, full height
        (Vec2::new(half_w, 0.0), Vec2::new(t, bar_h)),
        (Vec2::new(-half_w, 0.0), Vec2::new(t, bar_h)),
    ];

    // Children of the hotspot, so they inherit its position and ride the room's
    // breathing motion with it. `with_children` on `EntityCommands` sets the
    // relationship, so there is no separate re-parenting step to get wrong.
    hotspot.with_children(|outlined| {
        for (at, extent) in bars {
            outlined.spawn((
                HotspotOutline,
                Sprite::from_color(OUTLINE_COLOR, extent),
                Transform::from_xyz(at.x, at.y, HOTSPOT_OUTLINE_Z - HOTSPOT_Z),
                // The bars must never take a click: the hotspot they sit on is the
                // thing that is clickable, and a pickable bar in front of it would
                // either swallow the click or shadow the door.
                Pickable::IGNORE,
                Visibility::Hidden,
            ));
        }
    });
}

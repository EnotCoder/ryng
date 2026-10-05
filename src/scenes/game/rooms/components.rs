use bevy::prelude::*;

use crate::acts::Item;

#[derive(Component, Clone, Copy, PartialEq)]
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

/// One of the four bars that draw a hotspot's outline.
///
/// A component rather than a tag on the hotspot itself, because the bars are
/// children: they ride along with the room's breathing motion and are despawned
/// with the room, and they are positioned relative to the hotspot rather than in
/// world coordinates.
#[derive(Component)]
pub struct HotspotOutline;

/// How thick the hover outline is drawn, in design pixels.
///
/// 6 rather than the 5 this started at: at 1:1 that is a 5px hairline, and the
/// room art is photographic, so a thin bright line on it is easy to lose against
/// the clutter - the first version was measurably invisible over a light door. The
/// bars overlap the hotspot's own rectangle by this much on every side, so the
/// outline traces the edge of the clickable area rather than sitting just inside
/// it.
pub const HOTSPOT_OUTLINE_THICKNESS: f32 = 6.0;

/// The white of the hover outline. A warm white would blend into the yellow lamps
/// in the rooms and into the white signage; plain white reads against everything
/// in the game.
pub const OUTLINE_COLOR: Color = Color::WHITE;

/// Above the hotspot it belongs to, which is picked at [`HOTSPOT_Z`], and below
/// an NPC's click target, which is the whole reason the ordering is a constant
/// rather than an inline `0.1`: the concierge's way out is a hotspot and the
/// granny stands in front of it, so an outline drawn over her target would eat
/// the clicks meant for her. See [`crate::scenes::game::npc`].
///
/// The bars are `Pickable::IGNORE` anyway, so they cannot take a click; this is
/// about what they cover, not what they receive.
pub const HOTSPOT_OUTLINE_Z: f32 = 1.25;

/// The size the blinking arrow is drawn at, whatever the hotspot underneath it
/// happens to be. It used to be derived from the hotspot rectangle, which made
/// the arrow grow with every door that was resized and gave it a different size
/// on the item spots than on the doors.
pub const HOTSPOT_ICON_SIZE: Vec2 = Vec2::splat(10.0);

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

#[derive(Component, Clone, Copy, PartialEq)]
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

//! Items that live in the world rather than the inventory.
//!
//! The teddy starts on the floor of the basement, can be carried, and can be
//! left in one of the three floor 2 apartments. Only one of those states is
//! true at a time, so it is a single value rather than a flag on the item plus a
//! flag on the room - the two cannot disagree.

use bevy::prelude::*;

use crate::acts::{Inventory, Item};
use crate::scenes::game::rooms::components::HotspotAction;

#[cfg(test)]
mod tests;

/// Where the teddy is right now.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Teddy {
    /// Lying on the floor of the room with this asset path.
    Lying(&'static str),
    /// In the player's inventory.
    Carried,
}

/// Marked on the teddy sprite so the sync system can find it again.
#[derive(Component)]
pub struct TeddySprite;

/// Marks the pickup and put-down hotspots, which are the only ones that come
/// and go with the item rather than being part of the room.
#[derive(Component)]
pub struct ItemHotspot;

/// The picture the teddy is drawn with, both on the floor and in the slot.
pub const TEDDY_TEXTURE: &str = "tex/ui/icons_inv/teddy.png";

/// On-screen size of the teddy. The source is 512px, drawn down to this.
pub const TEDDY_SIZE: f32 = 130.0;

/// The first slot with nothing in it.
pub fn first_free_slot(inventory: &Inventory) -> Option<usize> {
    inventory.0.iter().position(Option::is_none)
}

/// Pick the teddy up from `room`.
///
/// Does nothing unless the teddy is lying in that exact room, or the inventory
/// is full - there is no message system to explain a refusal.
pub fn try_take(teddy: &mut Teddy, room: &'static str, inventory: &mut Inventory) -> bool {
    if *teddy != Teddy::Lying(room) {
        return false;
    }
    let Some(slot) = first_free_slot(inventory) else {
        return false;
    };
    inventory.0[slot] = Some(Item::Teddy);
    *teddy = Teddy::Carried;
    true
}

/// Put the held teddy down in `room`, emptying the active slot.
pub fn try_drop(
    teddy: &mut Teddy,
    room: &'static str,
    inventory: &mut Inventory,
    active_slot: usize,
) -> bool {
    if *teddy != Teddy::Carried {
        return false;
    }
    if inventory.0.get(active_slot).copied().flatten() != Some(Item::Teddy) {
        return false;
    }
    inventory.0[active_slot] = None;
    *teddy = Teddy::Lying(room);
    true
}

/// Runs `attempt` and plays a sound when it reports a change, so pickup,
/// putting down and handing over a card all sound the same.
pub(crate) fn act(commands: &mut Commands, asset_server: &AssetServer, done: bool) {
    if done {
        crate::scenes::sound::play_item_sound(commands, asset_server);
    }
}

/// Whether the teddy should be drawn in the room with this path.
pub fn should_show_in(teddy: &Teddy, room: &str) -> bool {
    matches!(teddy, Teddy::Lying(where_) if *where_ == room)
}

/// Whether a pickup or put-down spot should exist in the room the player is
/// standing in.
///
/// A pickup spot only exists where the teddy actually is, and a put-down spot
/// only while it is being carried. So leaving it in an apartment removes the
/// spot there, because the teddy is no longer in hand, and in the other two,
/// because the teddy is elsewhere.
pub fn spot_is_live(teddy: &Teddy, here: &str, action: &HotspotAction) -> bool {
    match action {
        HotspotAction::Take(_) => should_show_in(teddy, here),
        HotspotAction::Drop(_) => matches!(teddy, Teddy::Carried),
        // Doors are part of the room and are never hidden.
        HotspotAction::GoToRoom(_) => true,
    }
}

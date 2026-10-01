//! Items that live in the world rather than the inventory.
//!
//! The teddy starts on the basement floor; the three tools are in the floor 2
//! apartments. Any of them can be picked up, and the teddy can be put down
//! again. An item is either resting in one room or in the player's inventory,
//! never both and never neither, so that is one map keyed by item rather than a
//! flag on the item plus a flag on the room.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::acts::{Inventory, Item};
use crate::scenes::game::rooms::components::HotspotAction;
use crate::scenes::game::rooms::data::RoomDef;

#[cfg(test)]
mod tests;

/// Items currently lying on the floor, and where.
#[derive(Resource, Default, Debug)]
pub struct WorldItems {
    resting: HashMap<Item, &'static str>,
}

impl WorldItems {
    /// The starting layout, as (item, room) pairs.
    pub fn with_resting(entries: impl IntoIterator<Item = (Item, &'static str)>) -> Self {
        Self {
            resting: entries.into_iter().collect(),
        }
    }

    /// Where an item is lying, if it is in the world at all.
    pub fn resting_in(&self, item: Item) -> Option<&'static str> {
        self.resting.get(&item).copied()
    }

    /// Every item currently lying on the floor, and where.
    pub fn iter(&self) -> impl Iterator<Item = (Item, &'static str)> {
        self.resting.iter().map(|(item, room)| (*item, *room))
    }

    fn lie_down(&mut self, item: Item, room: &'static str) {
        self.resting.insert(item, room);
    }

    fn pick_up(&mut self, item: Item) {
        self.resting.remove(&item);
    }
}

/// Marks a sprite as the world picture of some item, so the sync system can tell
/// which item a stale sprite belonged to.
#[derive(Component)]
pub struct ItemSprite(pub Item);

/// Marks the pickup and put-down hotspots, which are the only ones that come
/// and go with an item rather than being part of the room.
#[derive(Component)]
pub struct ItemHotspot;

/// On-screen size of an item lying on the floor. The source icons are 100px, so
/// this is a downscale and stays sharp; drawing them at room-item size would
/// blur a small icon over photographic walls.
pub const ITEM_SIZE: f32 = 80.0;

/// The first slot with nothing in it.
pub fn first_free_slot(inventory: &Inventory) -> Option<usize> {
    inventory.0.iter().position(Option::is_none)
}

/// Pick `item` up from `room`.
///
/// Does nothing unless it is lying in that exact room, or the inventory is full
/// - there is no message system to explain a refusal.
pub fn try_take(
    world: &mut WorldItems,
    item: Item,
    room: &'static str,
    inventory: &mut Inventory,
) -> bool {
    if world.resting_in(item) != Some(room) {
        return false;
    }
    let Some(slot) = first_free_slot(inventory) else {
        return false;
    };
    inventory.0[slot] = Some(item);
    world.pick_up(item);
    true
}

/// Put the held `item` down in `room`, emptying the active slot.
pub fn try_drop(
    world: &mut WorldItems,
    item: Item,
    room: &'static str,
    inventory: &mut Inventory,
    active_slot: usize,
) -> bool {
    // If it is already lying somewhere it is not in hand, so there is nothing
    // to drop and it must not be duplicated into this room.
    if world.resting_in(item).is_some() {
        return false;
    }
    if inventory.0.get(active_slot).copied().flatten() != Some(item) {
        return false;
    }
    inventory.0[active_slot] = None;
    world.lie_down(item, room);
    true
}

/// Runs `attempt` and plays a sound when it reports a change, so pickup,
/// putting down and handing over a card all sound the same.
pub(crate) fn act(commands: &mut Commands, asset_server: &AssetServer, done: bool) {
    if done {
        crate::scenes::sound::play_item_sound(commands, asset_server);
    }
}

/// Whether `item` should be drawn in the room with this path.
pub fn should_show_in(world: &WorldItems, item: Item, room: &str) -> bool {
    world.resting_in(item) == Some(room)
}

/// Whether a pickup or put-down spot should exist in the room the player is
/// standing in.
///
/// A pickup spot only exists where the item actually is, and a put-down spot
/// only while it is carried. So leaving the teddy in a flat removes the spot
/// there, because it is no longer in hand, and in the other two, because it is
/// elsewhere.
pub fn spot_is_live(world: &WorldItems, here: &str, action: &HotspotAction) -> bool {
    match action {
        HotspotAction::Take(item) => should_show_in(world, *item, here),
        HotspotAction::Drop(item) => world.resting_in(*item).is_none(),
        // Doors are part of the room and are never hidden.
        HotspotAction::GoToRoom(_) => true,
    }
}

/// Where an item is drawn in a room: the position of that room's own pickup
/// hotspot for the item.
///
/// Reading the position back out of the table rather than keeping a second list
/// of coordinates is what stops the sprite and the clickable area from drifting
/// apart - there is only ever one number to edit.
pub fn item_position(def: &RoomDef, item: Item) -> Option<Vec2> {
    def.variants[0]
        .hotspots
        .iter()
        .find(|hotspot| matches!(hotspot.action, HotspotAction::Take(which) if which == item))
        .map(|hotspot| hotspot.pos)
}

/// Whether the inventory holds every one of `needed`.
///
/// A door that asks for several items checks them as a set, and does not consume
/// them: the tools stay in the bag.
pub fn holds_all(inventory: &Inventory, needed: &[Item]) -> bool {
    let bag: Vec<Item> = inventory.0.iter().filter_map(|slot| *slot).collect();
    needed.iter().all(|item| bag.contains(item))
}

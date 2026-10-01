//! Room table: one static row per room, looked up by asset path.
//!
//! Adding a room = one row in [`table`]. Adding an asset = one constant in
//! [`p`]. `all_paths` is derived from the table, so the preload list can no
//! longer drift out of sync with the rooms that actually exist.
//!
//! Layout:
//! - [`p`] - asset paths
//! - [`builders`] - the `room!` / `hop!` family that writes the rows
//! - [`table`] - the rows themselves
//! - `tests` - module tests, `#[cfg(test)]` only

use bevy::prelude::*;

use crate::acts::ActId;
use crate::scenes::game::rooms::components::RoomVariant;
use crate::scenes::sound::{Music, TransitionSound};

mod builders;
pub(crate) mod p;
mod table;

#[cfg(test)]
mod tests;

use table::{ROOMS, UNKNOWN};

#[derive(Component, Clone, Copy)]
pub(crate) struct RoomDef {
    pub sound: TransitionSound,
    pub music: Music,
    pub interactive: bool,
    pub auto_next: Option<(&'static str, f32)>,
    pub variants: &'static [RoomVariant],
    pub next_act: Option<ActId>,
}

/// `act` is the act the player is currently in: the concierge asks for the pass
/// and the elevator still works during act 1, after the basement loop it does
/// not.
pub(crate) fn room_def(path: &'static str, act: ActId) -> RoomDef {
    let key = if act != ActId::ActOne && path == p::F1_CONCIERGE {
        p::F1_CONCIERGE_DARK
    } else {
        path
    };
    ROOMS
        .iter()
        .find(|room| room.variants[0].path == key)
        .copied()
        .unwrap_or(UNKNOWN)
}

/// The rectangle `hop!` gives a door when no size is given. `describe` compares
/// against it so it can leave the argument out of the line it prints.
#[cfg(feature = "hotspot-editor")]
pub(crate) fn door_size_default() -> Vec2 {
    builders::HOTSPOT_SIZE
}

/// Every room key, in table order.
///
/// The keys are the first variant of each row; a secondary carousel variant has
/// no row of its own and would look up as `UNKNOWN`, so those are not listed.
#[cfg(feature = "hotspot-editor")]
pub(crate) fn room_keys() -> impl Iterator<Item = &'static str> {
    ROOMS.iter().map(|def| def.variants[0].path)
}

/// `room_keys` as an indexable list, for stepping through rooms.
#[cfg(feature = "hotspot-editor")]
pub(crate) fn room_key_list() -> Vec<&'static str> {
    room_keys().collect()
}

/// Every picture the game needs, for the loading overlay.
pub(crate) fn all_paths() -> impl Iterator<Item = &'static str> {
    ROOMS
        .iter()
        .flat_map(|room| room.variants.iter())
        .map(|variant| variant.path)
}

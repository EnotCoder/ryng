//! Room table: one static row per room, looked up by asset path.
//!
//! Adding a room = one row in the act file it belongs to. Adding an asset = one
//! constant in [`p`]. `all_paths` is derived from the table, so the preload list
//! can no longer drift out of sync with the rooms that actually exist.
//!
//! The rows live one file per act under [`table`], and `ACTS` there lists the
//! acts in play order. Everything in this module reads the rooms through
//! [`rooms`] rather than through a single flat slice, so adding an act file is the
//! only thing needed to grow the game.
//!
//! Layout:
//! - [`p`] - asset paths
//! - [`builders`] - the `room!` / `hop!` family that writes the rows
//! - [`table`] - the rows themselves, one file per act
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

use table::{ACTS, UNKNOWN};

/// Every room in the table, acts in play order.
///
/// The table is a list of lists so each act can be its own file; this is the one
/// place that flattens it. The order is the player's route, which the hotspot
/// editor's room stepping depends on, so it is the `ACTS` order in [`table`].
pub(crate) fn rooms() -> impl Iterator<Item = &'static RoomDef> {
    ACTS.iter().flat_map(|act| act.iter())
}

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
    rooms()
        .find(|room| room.variants[0].path == key)
        .copied()
        .unwrap_or(UNKNOWN)
}

/// Every picture the game needs, for the loading overlay.
///
/// The NPC pictures are included: an NPC's texture is otherwise only asked for when
/// the player walks into its room, which is exactly the moment the overlay is gone
/// and the sprite would pop in over the fade.
pub(crate) fn all_paths() -> impl Iterator<Item = &'static str> {
    rooms()
        .flat_map(|room| room.variants.iter())
        .map(|variant| variant.path)
        .chain(crate::scenes::game::npc::paths())
}

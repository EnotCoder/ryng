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
/// place that flattens it. The order is the player's route, which is what the
/// `--rooms` flag in [`crate::cli`] counts along.
pub(crate) fn rooms() -> impl Iterator<Item = &'static RoomDef> {
    ACTS
        .iter()
        .flat_map(|(_, rows)| rows.iter())
}

/// The same list as [`rooms`], each row paired with the act that owns it.
///
/// A room belongs to the act the player is in while they stand in it, so starting
/// the game in one without its act would put `CurrentAct` somewhere the route
/// never does - and act 2 is what decides whether the concierge is lit.
pub(crate) fn rooms_with_act() -> impl Iterator<Item = (ActId, &'static RoomDef)> {
    ACTS.iter()
        .flat_map(|(act, rows)| rows.iter().map(move |room| (*act, room)))
}

/// The act that owns `path`, if the table has a row for it.
pub(crate) fn act_of(path: &str) -> Option<ActId> {
    ACTS.iter()
        .find(|(_, rows)| rows.iter().any(|room| key_of(room) == path))
        .map(|(act, _)| *act)
}

/// A room's lookup key: the path of its first variant.
///
/// The key rather than the whole row, because that is what every caller that
/// wants "which room is this" actually has to hand onwards.
pub(crate) fn key_of(room: &RoomDef) -> &'static str {
    room.variants[0].path
}

/// The shot the carousel control leads to from `index`: the next one along, wrapping.
///
/// `None` when there is nowhere to go - a single shot, or an empty room, and the
/// modulo below would otherwise divide by zero.
///
/// One control rather than a pair, so there is no "back": it walks the carousel the
/// same way every time. With the two shots every carousel in the game has that is
/// simply the other one, which is what the control's picture promises the player.
/// A carousel with three or more shots could not be walked back this way, and would
/// need a direction here again.
pub(crate) fn control_target(index: usize, count: usize) -> Option<usize> {
    (count > 1).then(|| (index + 1) % count)
}

#[derive(Component, Clone, Copy)]
pub(crate) struct RoomDef {
    pub sound: TransitionSound,
    pub music: Music,
    pub interactive: bool,
    pub auto_next: Option<(&'static str, f32)>,
    pub variants: &'static [RoomVariant],
    pub next_act: Option<ActId>,
    /// The pan played instead of a cut when the player flips between this room's
    /// shots. `None` for a room with one shot and for a carousel that has no pan
    /// drawn - a cut is the right answer there, not an animation of nothing.
    ///
    /// The list is the direction the art was drawn in, from the first shot towards
    /// the last, and the flip plays it backwards when the player goes the other
    /// way, so one list serves both arrows.
    pub flip: Option<&'static [&'static str]>,
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
///
/// The flip frames are included for the same reason and more sharply. They are
/// asked for while the player is already standing in the room, one arrow press
/// after the overlay has gone - and an arrow press is a click, so a frame that has
/// not finished loading lands as a blank rectangle in the middle of the pan.
///
/// The carousel control is in the same position for the same reason: it is asked
/// for the frame the room opens on, and a control with no picture on it is a
/// rectangle the player cannot read.
pub(crate) fn all_paths() -> impl Iterator<Item = &'static str> {
    rooms()
        .flat_map(|room| {
            room.variants
                .iter()
                .map(|variant| variant.path)
                .chain(variant_previews(room))
                .chain(room.flip.unwrap_or(&[]).iter().copied())
        })
        .chain(crate::scenes::game::npc::paths())
}

/// The carousel controls a room's shots can be reached with.
pub(crate) fn variant_previews(room: &RoomDef) -> impl Iterator<Item = &'static str> {
    room.variants
        .iter()
        .filter_map(|variant| variant.preview)
}

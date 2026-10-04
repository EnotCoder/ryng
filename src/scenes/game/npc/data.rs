//! The NPC table.
//!
//! An NPC is a character standing in a room: a sprite, a rectangle you can click,
//! and the lines she says when you do. Like the room table, it is data rather
//! than code, so a new character is one row here and one picture.
//!
//! Placement is authored by hand in world coordinates, the same units as the
//! hotspots: `x` from the centre of the room, `y` up. The rooms are 1280x720 and
//! the camera is `FixedVertical`, so the numbers can be measured off the art.

use bevy::prelude::*;

/// Who a character is.
///
/// An enum rather than a `&'static str` because the dialogue box needs a name to
/// put above the line, and a key is not a name the player should read.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NpcId {
    Granny,
}

impl NpcId {
    /// What the dialogue box calls her.
    pub fn name(self) -> &'static str {
        match self {
            NpcId::Granny => "Granny",
        }
    }
}

/// Everything about one character.
#[derive(Clone, Copy)]
pub struct NpcDef {
    pub id: NpcId,

    pub texture: &'static str,

    pub room: &'static str,

    pub pos: Vec2,

    pub size: Vec2,

    pub hit: Vec2,

    pub lines: &'static [&'static str],
}

use crate::scenes::game::rooms::data::p;

pub const GRANNY: NpcDef = NpcDef {
    id: NpcId::Granny,
    texture: "tex/npc/granny.png",
    room: p::F1_CONCIERGE,

    pos: Vec2::new(-300.0, 0.0),

    size: Vec2::new(180.0, 300.0),

    hit: Vec2::new(250.0, 410.0),

    lines: &[
        "You did not ride the elevator.\nIt has not worked in this building for years.",
        "I have been sitting at this desk for thirty years.\nThe elevator fell exactly once, and that was you.",
    ],
};

pub(crate) static NPCS: &[NpcDef] = &[GRANNY];

/// Every character, in table order.
pub(crate) fn npcs() -> impl Iterator<Item = &'static NpcDef> {
    NPCS.iter()
}

/// The character with this id.
pub(crate) fn npc(id: NpcId) -> &'static NpcDef {
    NPCS.iter()
        .find(|def| def.id == id)
        .expect("an npc in the table")
}

/// Every NPC picture, for the loading overlay.
pub(crate) fn paths() -> impl Iterator<Item = &'static str> {
    NPCS.iter().map(|def| def.texture)
}

/// What the dialogue box calls this character.
pub fn display_name(id: NpcId) -> &'static str {
    id.name()
}

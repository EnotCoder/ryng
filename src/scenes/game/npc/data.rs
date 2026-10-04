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
    /// Her picture.
    pub texture: &'static str,
    /// Which room she stands in, by that room's key.
    ///
    /// The concierge has two rows - lit and dark - and this names only the lit
    /// one. That is deliberate and is the whole point of her being here: in the
    /// dark row nobody is on duty, so a granny sitting at an empty desk would be
    /// a bug rather than a surprise.
    pub room: &'static str,
    /// Where her sprite is centred.
    pub pos: Vec2,
    /// The drawn size of the whole 256x384 canvas, transparent margin included.
    pub size: Vec2,
    /// The clickable rectangle, in the same place as `size`.
    ///
    /// Wider and taller than the figure on purpose: the sprite is a person seen
    /// through a hatch in the desk, so most of her pixels are dark coat against
    /// dark background and the player aims at the shape, not at a tight box.
    pub hit: Vec2,
    /// What she says, in order, one line per conversation. The last line repeats
    /// once they run out.
    pub lines: &'static [&'static str],
}

use crate::scenes::game::rooms::data::p;

pub const GRANNY: NpcDef = NpcDef {
    id: NpcId::Granny,
    texture: "tex/npc/granny.png",
    room: p::F1_CONCIERGE,
    // She stands at the right-hand end of the hatch in the desk, which is wider than
    // the doorway hotspot: the recess runs to about x = 240 while the door stops at
    // x = 100, so there is room for her beside it rather than on top of it.
    //
    // The way out of this room is a 200x300 hotspot across the middle. Her click
    // target is drawn above the hotspot layer, so any overlap means the player
    // clicks the door and she talks instead, and they cannot leave - a soft lock
    // that is invisible until you try to walk out. `tests::she_does_not_cover_the_
    // way_out_of_the_room` keeps that true; move her back towards the middle and it
    // fails.
    pos: Vec2::new(170.0, 105.0),
    // Drawn large enough to read as a person rather than a detail: 120x200 against
    // the 1280x720 room. Her feet land near the counter at world y = 60 and her head
    // is still inside the recess, so she reads as standing behind the desk.
    size: Vec2::new(120.0, 200.0),
    // Wider than the figure so the target is forgiving, and kept clear of the door:
    // 170 - 65 = 105, which is 5px past the door's right edge at 100.
    hit: Vec2::new(130.0, 215.0),
    // The warning the whole character exists to give: the player walks past the
    // elevator and the ride is what drops them into the basement. She says it once
    // and then stops repeating it, which is the point - missed it the first time,
    // click her again.
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

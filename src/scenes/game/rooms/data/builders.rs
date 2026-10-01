//! Room builders.
//!
//! These are macros rather than `const fn`s on purpose. A `&[..]` can only be
//! promoted to `'static` when it is written syntactically where the borrow
//! happens: a `const fn` that fills the array from its own parameters would
//! produce a temporary that dies with the frame (E0716). Expanding the macro at
//! the `static` puts every literal straight into the constant initializer.
//!
//! The macros are re-exported with `pub(crate) use` so `table` can pull them in
//! by name. That keeps them out of the crate root, which `#[macro_export]` would
//! have done.

use bevy::prelude::*;

pub(crate) const HOTSPOT_SIZE: Vec2 = Vec2::new(200.0, 300.0);

/// An item lying on the floor is a smaller target than a doorway.
pub(crate) const ITEM_HOTSPOT_SIZE: Vec2 = Vec2::new(140.0, 140.0);

/// A hotspot leading to `target`.
///
/// The size defaults to a door-sized rectangle; pass one to override it, which
/// is what a hotspot low on the picture needs so it does not run off the bottom
/// of the room.
macro_rules! hop {
    ($target:expr, $x:expr, $y:expr) => {
        hop!($target, $x, $y, HOTSPOT_SIZE)
    };
    ($target:expr, $x:expr, $y:expr, $size:expr) => {
        HotspotDef {
            action: HotspotAction::GoToRoom($target),
            pos: Vec2::new($x, $y),
            size: $size,
            gate: None,
            require: &[],
        }
    };
}
pub(crate) use hop;

/// A hotspot that only opens if the player holds `$item` in the active slot.
macro_rules! gated {
    ($target:expr, $x:expr, $y:expr, $item:expr) => {
        HotspotDef {
            action: HotspotAction::GoToRoom($target),
            pos: Vec2::new($x, $y),
            size: HOTSPOT_SIZE,
            gate: Some($item),
            require: &[],
        }
    };
}
pub(crate) use gated;

macro_rules! shot {
    ($path:expr, $title:expr, $story:expr, $hotspots:expr) => {
        RoomVariant {
            path: $path,
            title: $title,
            story: $story,
            hotspots: $hotspots,
        }
    };
}
pub(crate) use shot;

/// One picture, one variant, player-driven. Covers most of the rooms.
macro_rules! room {
    ($path:expr, $title:expr, $story:expr, $sound:expr, $music:expr, $hotspots:expr) => {
        RoomDef {
            sound: $sound,
            music: $music,
            interactive: true,
            auto_next: None,
            variants: &[shot!($path, $title, $story, $hotspots)],
            next_act: None,
        }
    };
}
pub(crate) use room;

/// A non-interactive beat: it plays, waits, then moves on by itself.
macro_rules! beat {
    ($path:expr, $title:expr, $story:expr, $sound:expr, $music:expr, $auto_next:expr) => {
        RoomDef {
            sound: $sound,
            music: $music,
            interactive: false,
            auto_next: $auto_next,
            variants: &[shot!($path, $title, $story, &[])],
            next_act: None,
        }
    };
}
pub(crate) use beat;

/// A beat that also advances the act counter.
macro_rules! chapter {
    ($path:expr, $title:expr, $story:expr, $music:expr, $auto_next:expr, $act:expr) => {
        RoomDef {
            sound: TransitionSound::None,
            music: $music,
            interactive: false,
            auto_next: $auto_next,
            variants: &[shot!($path, $title, $story, &[])],
            next_act: Some($act),
        }
    };
}
pub(crate) use chapter;

/// Two or more pictures the player flips between.
macro_rules! carousel {
    ($sound:expr, $music:expr, $($variant:expr),+ $(,)?) => {
        RoomDef {
            sound: $sound,
            music: $music,
            interactive: true,
            auto_next: None,
            variants: &[$($variant),+],
            next_act: None,
        }
    };
}
pub(crate) use carousel;

/// Pick an item up. The sprite itself is the affordance, so `spawn_room_content`
/// skips the blinking chevron on these.
macro_rules! take {
    ($item:expr, $x:expr, $y:expr) => {
        HotspotDef {
            action: HotspotAction::Take($item),
            pos: Vec2::new($x, $y),
            size: ITEM_HOTSPOT_SIZE,
            gate: None,
            require: &[],
        }
    };
}
pub(crate) use take;

/// Put the held item down here.
macro_rules! drop {
    ($item:expr, $x:expr, $y:expr) => {
        HotspotDef {
            action: HotspotAction::Drop($item),
            pos: Vec2::new($x, $y),
            size: ITEM_HOTSPOT_SIZE,
            gate: None,
            require: &[],
        }
    };
}
pub(crate) use drop;

/// A door that only opens once every one of `$items` is in the bag. None of them
/// is spent, so the player can still walk back through afterwards. `$items` is a
/// slice rather than a list, so a set can be named at the use site -
/// `Item::DOOR_TOOLS` for the three tools on the floor 2 door.
macro_rules! locked {
    ($target:expr, $x:expr, $y:expr, $size:expr, $items:expr) => {
        HotspotDef {
            action: HotspotAction::GoToRoom($target),
            pos: Vec2::new($x, $y),
            size: $size,
            gate: None,
            require: $items,
        }
    };
}
pub(crate) use locked;

/// A floor 2 apartment: a way out, a tool lying on the floor, and somewhere to
/// leave or retrieve the teddy.
///
/// The exit takes a size like `hop!` does, because the door in each apartment
/// sits in a different place on its picture and one rectangle does not fit all
/// three. Note the size comes straight after `$exit_y`, not after the teddy's
/// spot: the argument list is positional, and a size dropped into the wrong slot
/// silently moves the teddy instead of resizing the door.
macro_rules! apartment {
    (
        $path:expr, $title:expr, $exit:expr, $exit_x:expr, $exit_y:expr, $exit_size:expr,
        $rest_x:expr, $rest_y:expr,
        $tool:expr, $tool_x:expr, $tool_y:expr
    ) => {
        room!(
            $path,
            $title,
            "",
            TransitionSound::NextRoom,
            Music::Indoors,
            &[
                hop!($exit, $exit_x, $exit_y, $exit_size),
                drop!(Item::Teddy, $rest_x, $rest_y),
                take!(Item::Teddy, $rest_x, $rest_y),
                take!($tool, $tool_x, $tool_y)
            ]
        )
    };
    (
        $path:expr, $title:expr, $exit:expr, $exit_x:expr, $exit_y:expr,
        $rest_x:expr, $rest_y:expr,
        $tool:expr, $tool_x:expr, $tool_y:expr
    ) => {
        apartment!(
            $path,
            $title,
            $exit,
            $exit_x,
            $exit_y,
            HOTSPOT_SIZE,
            $rest_x,
            $rest_y,
            $tool,
            $tool_x,
            $tool_y
        )
    };
}
pub(crate) use apartment;

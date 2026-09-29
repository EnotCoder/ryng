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

/// A hotspot leading to `target`.
macro_rules! hop {
    ($target:expr, $x:expr, $y:expr) => {
        HotspotDef {
            action: HotspotAction::GoToRoom($target),
            pos: Vec2::new($x, $y),
            size: HOTSPOT_SIZE,
            gate: None,
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
    ($path:expr, $title:expr, $story:expr, $sound:expr, $hotspots:expr) => {
        RoomDef {
            sound: $sound,
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
    ($path:expr, $title:expr, $story:expr, $sound:expr, $auto_next:expr) => {
        RoomDef {
            sound: $sound,
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
    ($path:expr, $title:expr, $story:expr, $auto_next:expr, $act:expr) => {
        RoomDef {
            sound: TransitionSound::None,
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
    ($sound:expr, $($variant:expr),+ $(,)?) => {
        RoomDef {
            sound: $sound,
            interactive: true,
            auto_next: None,
            variants: &[$($variant),+],
            next_act: None,
        }
    };
}
pub(crate) use carousel;

/// Floor 2 side rooms: one door back to the hall, no story.
macro_rules! side_room {
    ($path:expr, $title:expr) => {
        room!(
            $path,
            $title,
            "",
            TransitionSound::NextRoom,
            &[hop!(p::F2_HALL, 0.0, 0.0)]
        )
    };
}
pub(crate) use side_room;

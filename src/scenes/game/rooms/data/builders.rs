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
    ($target:expr, $x:expr, $y:expr, $size:expr, $item:expr) => {
        HotspotDef {
            action: HotspotAction::GoToRoom($target),
            pos: Vec2::new($x, $y),
            size: $size,
            gate: Some($item),
            require: &[],
        }
    };
}
pub(crate) use gated;

/// One picture of a room: the shot itself.
///
/// The four-argument form is a shot the carousel cannot reach, which is every shot
/// of a single-picture room. The five-argument form is a shot that can be reached
/// with the carousel control and carries the art for that control - the preview goes
/// last because it is the exception, and the hall's two shots are the only place in
/// the table that has one.
macro_rules! shot {
    ($path:expr, $title:expr, $story:expr, $hotspots:expr $(,)?) => {
        shot!(@build $path, $title, $story, $hotspots, None)
    };
    ($path:expr, $title:expr, $story:expr, $hotspots:expr, $preview:expr $(,)?) => {
        shot!(@build $path, $title, $story, $hotspots, Some($preview))
    };
    (@build $path:expr, $title:expr, $story:expr, $hotspots:expr, $preview:expr) => {
        RoomVariant {
            path: $path,
            title: $title,
            story: $story,
            hotspots: $hotspots,
            preview: $preview,
        }
    };
}
pub(crate) use shot;

/// The frames of a shot-to-shot flip, in play order.
///
/// `$dir` is a literal rather than a `p` constant because `concat!` will not take
/// one, and twelve constants per animation is worse than naming the folder once.
/// The folder is therefore not in `p` either - the paths written here are the only
/// thing that names it, and `every_texture_exists` reads the same table.
///
/// Written in the direction the art was drawn in, from the first shot of the room
/// towards the last. The flip plays the list backwards when the player goes the
/// other way, so there is one list and not two.
macro_rules! flip {
    ($dir:literal, $($frame:literal),+ $(,)?) => {
        Some(&[$(concat!($dir, "/", $frame)),+])
    };
}
pub(crate) use flip;

/// The frames of a room that plays its own pictures while it stands.
///
/// Identical in shape to [`flip!`], and for the same reason: `concat!` will not
/// take a `p` constant, so the folder is a literal here and not in `p`.
macro_rules! anim {
    ($dir:literal, $($frame:literal),+ $(,)?) => {
        Some(&[$(concat!($dir, "/", $frame)),+])
    };
}
pub(crate) use anim;

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
            flip: None,
            anim: None,
        }
    };
}
pub(crate) use room;

/// A non-interactive beat: it plays, waits, then moves on by itself.
///
/// The form taking an [`anim!`] is a beat whose picture is a short film rather
/// than one frame - the lift falling is the only one. The animation arm comes
/// first and matches on the `anim` name for the same reason `carousel!` does on
/// `flip`: both expand to an expression, and a bare `$auto_next` arm written
/// above would swallow the film as one more argument and quietly build a beat
/// that never plays it.
macro_rules! beat {
    ($path:expr, $title:expr, $story:expr, $sound:expr, $music:expr, anim ! ($dir:literal, $($frame:literal),+ $(,)?), $auto_next:expr $(,)?) => {
        RoomDef {
            sound: $sound,
            music: $music,
            interactive: false,
            auto_next: $auto_next,
            variants: &[shot!($path, $title, $story, &[])],
            next_act: None,
            flip: None,
            anim: anim!($dir, $($frame),*),
        }
    };
    ($path:expr, $title:expr, $story:expr, $sound:expr, $music:expr, $auto_next:expr) => {
        RoomDef {
            sound: $sound,
            music: $music,
            interactive: false,
            auto_next: $auto_next,
            variants: &[shot!($path, $title, $story, &[])],
            next_act: None,
            flip: None,
            anim: None,
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
            flip: None,
            anim: None,
        }
    };
}
pub(crate) use chapter;

/// Two or more pictures the player flips between, cut straight to the next one.
///
/// The form that takes a [`flip!`] plays that pan between the shots instead of
/// cutting. The list belongs to the room whose first and last shots the art runs
/// between, because that is the direction it is played in.
///
/// The flip arm comes first and matches the `flip` name itself rather than
/// wrapping the list in brackets. `[..]` is a perfectly good expression, so a
/// bracketed form would be swallowed by the plain arm as one more `shot!` and build
/// a room whose last "picture" is the animation; and an `expr` fragment cannot be
/// followed by `]` at all, so the bracketed arm would never match in the first
/// place. Matching the name is what actually tells the two apart.
macro_rules! carousel {
    ($sound:expr, $music:expr, flip ! ($dir:literal, $($frame:literal),+ $(,)?), $($variant:expr),+ $(,)?) => {
        RoomDef {
            sound: $sound,
            music: $music,
            interactive: true,
            auto_next: None,
            variants: &[$($variant),+],
            next_act: None,
            flip: flip!($dir, $($frame),*),
            anim: None,
        }
    };
    ($sound:expr, $music:expr, $($variant:expr),+ $(,)?) => {
        RoomDef {
            sound: $sound,
            music: $music,
            interactive: true,
            auto_next: None,
            variants: &[$($variant),+],
            next_act: None,
            flip: None,
            anim: None,
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

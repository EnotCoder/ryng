//! The room table, one file per act.
//!
//! Each act file holds its own `ROOMS`, and the list below is the single place
//! that says what order the acts go in. That order is not cosmetic: the hotspot
//! editor's `[` and `]` step through `ACTS` in sequence and its readout counts
//! the rooms, so reordering this list moves the editor along with the table.
//!
//! A room belongs to the act that is current *while the player is standing in it*.
//! The act changes on entry to the room that carries `next_act`, so that room is
//! the first row of the next act's file:
//! - `B_HALL` carries `next_act: ActTwo`, so it opens `act_two`.
//! - `F2_HALL` carries `next_act: ActThree`, so it opens `act_three`.
//!
//! The elevator beat therefore belongs to act 1: the fall is the end of the ride
//! the player started in act 1, and `B_HALL` is what they arrive at.

use crate::acts::ActId;
use crate::scenes::sound::{Music, TransitionSound};

use super::RoomDef;
// A glob: `beat!` expands into `shot!`, and `RoomVariant` has to be nameable here
// for that expansion to resolve. The hotspot components are not needed - `beat!`
// writes no hotspots.
use super::builders::*;
use crate::scenes::game::rooms::components::RoomVariant;

mod act_one;
mod act_three;
mod act_two;

use act_one::ROOMS as ACT_ONE;
use act_three::ROOMS as ACT_THREE;
use act_two::ROOMS as ACT_TWO;

/// Every act's rows, in play order, each labelled with the act it belongs to.
///
/// The label is written out rather than inferred from the position, so adding an
/// act is a new row here and nowhere else. A `match` on the index would have to
/// decide what a fourth act is called, and `#[derive]`-ing an index into an
/// `ActId` would quietly call it `ActThree`.
///
/// The order is still the player's route and still what `rooms()` flattens, so it
/// is what the `--rooms` flag counts along.
pub(super) static ACTS: &[(ActId, &[RoomDef])] = &[
    (ActId::ActOne, ACT_ONE),
    (ActId::ActTwo, ACT_TWO),
    (ActId::ActThree, ACT_THREE),
];

/// The fallback for a path that is in no act: `room_def` hands this back rather
/// than panicking, so a typo in a `p` constant lands the player somewhere rather
/// than crashing. The main menu background doubles as the "nothing here" picture.
///
/// It lives here rather than in an act file because it belongs to no act.
pub(super) static UNKNOWN: RoomDef = beat!(
    "tex/main_fon.png",
    "Unknown room",
    "",
    TransitionSound::None,
    Music::Indoors,
    None
);

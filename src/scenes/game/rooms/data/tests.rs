//! Tests for the room table. The parent module gates this file behind
//! `#[cfg(test)]`, so it never reaches a release build.

use std::collections::HashSet;

use crate::acts::ActId;
use crate::scenes::game::rooms::components::HotspotAction;

use super::table::ROOMS;
use super::{p, room_def};

fn is_room(path: &str) -> bool {
    ROOMS.iter().any(|room| room.variants[0].path == path)
}

/// Nothing may link to a room that is not in the table: `room_def` would
/// silently hand back `UNKNOWN` and the player would land on a black screen.
#[test]
fn every_link_resolves() {
    for room in ROOMS {
        let links = room
            .variants
            .iter()
            .flat_map(|variant| variant.hotspots.iter())
            .map(|hotspot| match hotspot.action {
                HotspotAction::GoToRoom(target) => target,
            })
            .chain(room.auto_next.map(|(target, _)| target));
        for target in links {
            assert!(
                is_room(target),
                "{} -> {target} is not a key in ROOMS",
                room.variants[0].path,
            );
        }
    }
}

/// The lookup takes the first match, so a duplicated key makes a dead row.
#[test]
fn keys_are_unique() {
    let keys: HashSet<_> = ROOMS.iter().map(|room| room.variants[0].path).collect();
    assert_eq!(keys.len(), ROOMS.len(), "two rows share a key");
}

/// The concierge is the only act-dependent room, and it is easy to invert.
#[test]
fn concierge_is_dark_after_act_one() {
    let lit = room_def(p::F1_CONCIERGE, ActId::ActOne);
    assert_eq!(lit.variants[0].path, p::F1_CONCIERGE);
    for act in [ActId::ActTwo, ActId::ActThree] {
        let dark = room_def(p::F1_CONCIERGE, act);
        assert_eq!(dark.variants[0].path, p::F1_CONCIERGE_DARK);
    }
}

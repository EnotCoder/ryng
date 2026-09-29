//! Tests for the room table. The parent module gates this file behind
//! `#[cfg(test)]`, so it never reaches a release build.

use std::collections::HashSet;

use crate::acts::{ActId, default_act};
use crate::scenes::game::rooms::components::HotspotAction;

use super::table::ROOMS;
use super::{RoomDef, all_paths, p, room_def};

/// Every way out of a room: each hotspot plus the auto-next hand-off.
fn edges_of(room: &RoomDef) -> impl Iterator<Item = &'static str> {
    room.variants
        .iter()
        .flat_map(|variant| variant.hotspots.iter())
        .map(|hotspot| match hotspot.action {
            HotspotAction::GoToRoom(target) => target,
        })
        .chain(room.auto_next.map(|(target, _)| target))
}

/// The lookup key of a room, which is the path of its first variant.
fn key_of(room: &RoomDef) -> &'static str {
    room.variants[0].path
}

fn is_room(path: &str) -> bool {
    ROOMS.iter().any(|room| key_of(room) == path)
}

/// Every key in the table.
fn keys() -> impl Iterator<Item = &'static str> {
    ROOMS.iter().map(key_of)
}

/// Rooms that are in the table but that nothing links to yet. Each one is work
/// in progress, so listing it here keeps `all_rooms_are_reachable` green while
/// still failing if a *new* room is left dangling.
const PENDING: &[&str] = &[
    // Act 3 has no entry point yet; the lobby is written but nothing leads to it.
    p::MY_FLOOR,
];

/// Textures on disk that no room shows yet - either unwired act 3 art or a
/// leftover. Listed so the test catches *new* orphans without failing on
/// content that is knowingly parked. `ap_3` left this list when the floor 2
/// corridor gained a door into it.
const PARKED_ASSETS: &[&str] = &[
    "tex/rooms/my_floor/door_my_home.png",
    "tex/rooms/my_floor/door_nighbor_home.png",
    "tex/rooms/my_floor/open_door_my_home.png",
];

/// Nothing may link to a room that is not in the table: `room_def` would
/// silently hand back `UNKNOWN` and the player would land on a black screen.
#[test]
fn every_link_resolves() {
    for room in ROOMS {
        for target in edges_of(room) {
            assert!(
                is_room(target),
                "{} -> {target} is not a key in ROOMS",
                key_of(room),
            );
        }
    }
}

/// The lookup takes the first match, so a duplicated key makes a dead row.
#[test]
fn keys_are_unique() {
    let unique: HashSet<_> = keys().collect();
    assert_eq!(unique.len(), ROOMS.len(), "two rows share a key");
}

/// Every picture the table names has to exist, or the room spawns as a blank
/// sprite. This is the cheapest guard against a typo in a path constant.
#[test]
fn every_texture_exists() {
    for path in all_paths() {
        let full = std::path::Path::new("assets").join(path);
        assert!(full.exists(), "missing asset: assets/{path}");
    }
}

/// A typo in a path is far more likely than a missing file, but an asset that
/// exists and is never referenced is content the player can never reach.
#[test]
fn unreferenced_assets_are_known() {
    let known: HashSet<_> = all_paths().collect();
    let mut orphans = Vec::new();
    walk_assets("assets/tex/rooms", &mut orphans);
    orphans
        .retain(|path| !known.contains(path.as_str()) && !PARKED_ASSETS.contains(&path.as_str()));
    orphans.sort();

    assert!(
        orphans.is_empty(),
        "these room textures are never shown:\n  {}",
        orphans.join("\n  "),
    );
}

/// The rooms a walk from the starting room can end up in, following every legal
/// transition in every act.
///
/// Expansion goes through `room_def` rather than the raw table, because the
/// concierge is a different row depending on the act: entering it in act 2 hands
/// back the dark row, and that row's exit is the one the player takes. Walking
/// the raw edges would report the dark concierge as unreachable when it is not.
fn reachable() -> HashSet<&'static str> {
    fn expand(key: &'static str) -> Vec<&'static str> {
        let mut out = Vec::new();
        for act in [ActId::ActOne, ActId::ActTwo, ActId::ActThree] {
            let def = room_def(key, act);
            out.push(key_of(&def));
            out.extend(edges_of(&def));
        }
        out
    }

    let start = default_act().start_room;
    let mut seen = HashSet::new();
    let mut stack = vec![start];
    while let Some(path) = stack.pop() {
        if !seen.insert(path) {
            continue;
        }
        stack.extend(expand(path));
    }
    seen
}

/// Walks the room graph from the first act's start room. A room that cannot be
/// reached is invisible content, which is easy to author and hard to notice.
#[test]
fn all_rooms_are_reachable() {
    let start = default_act().start_room;
    assert!(
        is_room(start),
        "the game starts in an unknown room: {start}"
    );

    let seen = reachable();
    let mut unreachable: Vec<&str> = keys()
        .filter(|key| !seen.contains(key))
        .filter(|key| !PENDING.contains(key))
        .collect();
    unreachable.sort();

    assert!(
        unreachable.is_empty(),
        "no way to walk from {start} to these rooms:\n  {}",
        unreachable.join("\n  "),
    );
}

/// Rooms listed as pending must really be unreachable, otherwise the list has
/// gone stale and the test above would be lying about them.
#[test]
fn pending_rooms_are_still_unreachable() {
    let seen = reachable();
    for pending in PENDING {
        assert!(
            !seen.contains(pending),
            "{pending} is now reachable - drop it from PENDING",
        );
    }
}

/// The concierge is the only act-dependent room, and it is easy to invert.
#[test]
fn concierge_is_dark_after_act_one() {
    let lit = room_def(p::F1_CONCIERGE, ActId::ActOne);
    assert_eq!(key_of(&lit), p::F1_CONCIERGE);
    for act in [ActId::ActTwo, ActId::ActThree] {
        let dark = room_def(p::F1_CONCIERGE, act);
        assert_eq!(key_of(&dark), p::F1_CONCIERGE_DARK);
    }
}

/// A path that is in no table row must fall back, not panic.
#[test]
fn unknown_path_falls_back() {
    let fallback = room_def("tex/rooms/nowhere.png", ActId::ActOne);
    assert_eq!(key_of(&fallback), "tex/main_fon.png");
}

/// Every act must start in a room the table knows about.
#[test]
fn every_act_starts_in_a_real_room() {
    for act in [ActId::ActOne, ActId::ActTwo, ActId::ActThree] {
        let start = crate::acts::get_act(act).start_room;
        assert!(is_room(start), "act {act:?} starts in unknown room {start}");
    }
}

fn walk_assets(dir: &str, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("assets directory exists") {
        let entry = entry.expect("readable directory entry");
        let path = entry.path();
        if path.is_dir() {
            let dir = path.to_str().expect("utf-8 path").to_owned();
            walk_assets(&dir, out);
        } else {
            let path = path.to_str().expect("utf-8 path");
            out.push(
                path.strip_prefix("assets/")
                    .expect("under assets/")
                    .to_owned(),
            );
        }
    }
}

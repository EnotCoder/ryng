//! Tests for the room table. The parent module gates this file behind
//! `#[cfg(test)]`, so it never reaches a release build.

use std::collections::HashSet;

use bevy::prelude::Vec2;

use crate::acts::{ActId, default_act};
use crate::scenes::game::rooms::components::HotspotAction;

use super::table::ROOMS;
use super::{RoomDef, all_paths, p, room_def};

/// Every way out of a room: each hotspot plus the auto-next hand-off.
///
/// Pickups and drops are not edges; only `GoToRoom` moves the player.
fn edges_of(room: &RoomDef) -> impl Iterator<Item = &'static str> {
    room.variants
        .iter()
        .flat_map(|variant| variant.hotspots.iter())
        .filter_map(|hotspot| match hotspot.action {
            HotspotAction::GoToRoom(target) => Some(target),
            HotspotAction::Take(_) | HotspotAction::Drop(_) => None,
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

// --------------------------------------------------------------- apartments

/// The three floor 2 apartments, and the hotspots the macro gives each one.
fn apartment_spots(room: &RoomDef) -> Vec<(&HotspotAction, Vec2, Vec2)> {
    room.variants[0]
        .hotspots
        .iter()
        .map(|spot| (&spot.action, spot.pos, spot.size))
        .collect()
}

fn apartment_named(key: &'static str) -> &'static RoomDef {
    ROOMS
        .iter()
        .find(|room| key_of(room) == key)
        .expect("a row")
}

/// `apartment!` takes its arguments positionally, and the teddy's spot sits right
/// after the exit's coordinates. A size pasted into that gap compiles and runs,
/// and silently relocates the teddy instead of resizing the door.
///
/// So: every teddy spot has to be inside the frame, which a value dropped into
/// `$rest_x`/`$rest_y` typically is not. This is the guard that would have caught
/// the size 150/610 landing there and parking the teddy off the bottom of the
/// picture.
#[test]
fn every_apartment_teddy_spot_is_inside_the_frame() {
    for key in [p::AP_1, p::AP_2, p::AP_3] {
        let room = apartment_named(key);
        let teddy: Vec<_> = apartment_spots(room)
            .into_iter()
            .filter(|(action, _, _)| {
                matches!(
                    action,
                    HotspotAction::Take(crate::acts::Item::Teddy)
                        | HotspotAction::Drop(crate::acts::Item::Teddy)
                )
            })
            .collect();

        assert_eq!(teddy.len(), 2, "{key} should have a take and a drop");
        for (_, pos, _) in &teddy {
            assert!(
                pos.x.abs() <= crate::FRAME_HALF.x && pos.y.abs() <= crate::FRAME_HALF.y,
                "{key}: teddy spot at {pos} is off the frame - is a hotspot size \
                 sitting in the teddy's slot?",
            );
        }
    }
}

/// The exit size has to reach the door. Without this the macro could keep
/// accepting a size argument and quietly drop it, which is the same class of
/// silent failure as the teddy moving: the line reads as though it applied.
#[test]
fn an_apartment_exit_can_be_given_its_own_size() {
    let room = apartment_named(p::AP_3);
    let exit = room.variants[0]
        .hotspots
        .iter()
        .find(|spot| matches!(spot.action, HotspotAction::GoToRoom(_)))
        .expect("an apartment has a way out");

    assert_eq!(
        exit.size,
        Vec2::new(150.0, 610.0),
        "the size written into the apartment! call never reached the exit hotspot"
    );
}

/// The size argument sits between the exit and the teddy, so its position in the
/// argument list is itself the thing to pin down: the teddy's own spot is the one
/// that has to stay put when a size is supplied.
#[test]
fn an_apartment_size_argument_does_not_disturb_the_teddy() {
    // Apartment 1 and 3 carry the teddy at different spots, and only apartment 3
    // passes an exit size. If the size had landed in the teddy's slot, the two
    // apartments would agree on it.
    let with_size = apartment_spots(apartment_named(p::AP_3));
    let without_size = apartment_spots(apartment_named(p::AP_1));

    let teddy_of = |spots: Vec<(&HotspotAction, Vec2, Vec2)>| {
        spots
            .into_iter()
            .find(|(action, _, _)| matches!(action, HotspotAction::Drop(_)))
            .map(|(_, pos, _)| pos)
            .expect("a drop spot")
    };
    assert_eq!(teddy_of(with_size), Vec2::new(-190.0, -190.0));
    assert_eq!(teddy_of(without_size), Vec2::new(-240.0, -170.0));
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

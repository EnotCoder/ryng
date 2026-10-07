//! Tests for the room table. The parent module gates this file behind
//! `#[cfg(test)]`, so it never reaches a release build.

use std::collections::HashSet;

use bevy::prelude::Vec2;

use crate::acts::{ActId, default_act};
use crate::scenes::game::rooms::components::HotspotAction;

use super::table::ACTS;
use super::{
    RoomDef, all_paths, key_of, p, room_def, rooms, rooms_with_act, variant_previews,
};

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

fn is_room(path: &str) -> bool {
    rooms().any(|room| key_of(room) == path)
}

/// Every key in the table.
fn keys() -> impl Iterator<Item = &'static str> {
    rooms().map(key_of)
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
    for room in rooms() {
        for target in edges_of(room) {
            assert!(
                is_room(target),
                "{} -> {target} is not a key in the table",
                key_of(room),
            );
        }
    }
}

/// The lookup takes the first match, so a duplicated key makes a dead row.
#[test]
fn keys_are_unique() {
    let unique: HashSet<_> = keys().collect();
    let all: Vec<_> = rooms().collect();
    assert_eq!(unique.len(), all.len(), "two rows share a key");
}

// ------------------------------------------------------- one file per act

/// The table is split across act files, and `ACTS` is what stitches them back
/// together. Nothing here checks that the split is *sensible*, only that it did
/// not lose or reorder anything on the way.
///
/// The order is the substance: the hotspot editor steps rooms with `[` and `]`
/// and prints "room 7/21", so moving a row between files silently renumbers the
/// editor even when the game plays identically. Pinning the route here is what
/// makes such a move a deliberate act rather than a diff nobody can explain.
#[test]
fn the_table_is_ordered_by_the_player_s_route() {
    let route: Vec<&str> = keys().collect();
    assert_eq!(
        route,
        vec![
            // Act 1: street, concierge, hall, then the lift.
            p::F1_STREET_1,
            p::F1_STREET_2,
            p::F1_CONCIERGE,
            p::F1_CONCIERGE_DARK,
            p::F1_HALL,
            p::F1_HALL_DEAD,
            p::ELEVATOR,
            // Act 2: the basement, out to the courtyard, then up the stairs.
            p::B_HALL,
            p::B_CORRIDOR,
            p::B_DEEP,
            p::B_EXIT,
            p::B_STREET_1,
            p::B_STREET_2,
            p::STAIRS_1,
            p::STAIRS_2,
            // Act 3: floor 2 and the three apartments.
            p::F2_HALL,
            p::F2_CORRIDOR,
            p::AP_1,
            p::AP_2,
            p::AP_3,
            // Parked.
            p::MY_FLOOR,
        ],
    );
}

/// An act file that is declared but empty, or one that is never declared, both
/// compile and both look fine. The count pins the split itself: three acts, and
/// the total the rest of the tests iterate over.
#[test]
fn the_table_is_split_into_three_acts() {
    assert_eq!(ACTS.len(), 3, "the act count changed");
    let all: Vec<_> = rooms().collect();
    assert_eq!(all.len(), 21, "a row went missing or arrived");
}

/// The act is a property of the room the player is standing in, so it changes on
/// entry to the room that carries `next_act` - which makes that room the first row
/// of the next act's file. Act 1 is the exception: it has nothing before it, so it
/// opens at the game's starting room.
///
/// Getting the boundary wrong shows nothing in play - the game plays identically
/// either way - and only misfiles an act-dependent lookup, which today means the
/// concierge. So it has to be asserted rather than read off the file layout.
#[test]
fn each_act_starts_at_the_room_that_changes_into_it() {
    let starts = [default_act().start_room, p::B_HALL, p::F2_HALL];
    for ((_, rows), start) in ACTS.iter().zip(starts) {
        let keys: Vec<&str> = rows.iter().map(key_of).collect();
        assert_eq!(
            keys.first().copied(),
            Some(start),
            "act does not begin at the room that carries next_act"
        );
    }

    // And those rooms really do carry the change, or the boundaries above are a
    // guess. Act 1 has no such room: it is the act the game starts in.
    assert_eq!(
        room_def(p::B_HALL, ActId::ActOne).next_act,
        Some(ActId::ActTwo)
    );
    assert_eq!(
        room_def(p::F2_HALL, ActId::ActTwo).next_act,
        Some(ActId::ActThree),
    );
}

/// `ACTS` now labels each act explicitly rather than leaving the label to be read
/// off the position, so the two have to agree - a label that drifts from its rows
/// would make `rooms_with_act` report an act the route never enters, and
/// `--rooms` would open a room under the wrong one.
#[test]
fn each_act_is_labelled_with_itself() {
    let labels: Vec<ActId> = ACTS.iter().map(|(act_id, _)| *act_id).collect();
    assert_eq!(
        labels,
        vec![ActId::ActOne, ActId::ActTwo, ActId::ActThree],
        "the labels do not read one, two, three down ACTS",
    );
}

/// And the labels have to line up with what each act's own start room says, which
/// is the one place the game itself agrees on which act is which.
#[test]
fn each_rows_act_matches_the_act_that_starts_there() {
    for ((act_id, rows), start) in ACTS.iter().zip([
        default_act().start_room,
        p::B_HALL,
        p::F2_HALL,
    ]) {
        let first = *rows.first().expect("an act has rooms");
        assert_eq!(key_of(&first), start, "unexpected first room for {act_id:?}");
        assert_eq!(
            crate::scenes::game::rooms::data::act_of(start),
            Some(*act_id),
            "{act_id:?} is labelled but {start} disagrees",
        );
    }
}

/// `rooms_with_act` is what `--rooms` counts along, so it has to be `rooms()` with
/// nothing added and nothing dropped.
#[test]
fn the_numbered_route_is_the_whole_route() {
    let numbered: Vec<&str> = rooms_with_act().map(|(_, room)| key_of(room)).collect();
    let plain: Vec<&str> = rooms().map(key_of).collect();
    assert_eq!(numbered, plain);
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
    rooms().find(|room| key_of(room) == key).expect("a row")
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

/// The pan frames are preloaded like any other picture.
///
/// A frame that has not finished loading when the arrow is pressed draws nothing,
/// and the room at that moment is nothing but that frame - the real picture and its
/// hotspots are already despawned. The player would get the backdrop instead of a
/// pan, once per visit, and no amount of waiting would fix it because the wait is
/// what triggered it. So the frames have to be in the same list the loading overlay
/// waits on.
#[test]
fn pan_frames_are_preloaded() {
    let paths: Vec<_> = all_paths().collect();

    for room in [p::F1_HALL, p::F1_HALL_DEAD] {
        let def = room_def(room, ActId::ActOne);
        for frame in def.flip.unwrap() {
            assert!(
                paths.contains(frame),
                "{frame} is not preloaded, so it draws nothing when the pan reaches it",
            );
        }
    }
}

/// Every shot of a carousel can be reached with the control, and no other shot can.
///
/// The control's picture is named on the shot it leads *to*, so a shot missing one is
/// a shot the player cannot get to from the carousel - and the button would show the
/// destination's picture over the wrong destination, or nothing at all. A shot of a
/// single-picture room is the other way round: it is not somewhere a carousel goes,
/// and a control art on it would claim otherwise.
#[test]
fn control_pictures_belong_to_carousel_shots_only() {
    for room in rooms() {
        let key = key_of(room);
        let is_carousel = room.variants.len() > 1;
        for (index, variant) in room.variants.iter().enumerate() {
            match variant.preview {
                Some(path) => assert!(
                    is_carousel,
                    "{key} has one shot and shot {index} ({}) carries a control \
                     picture ({path}), so the control would claim a shot the room \
                     cannot reach",
                    variant.path,
                ),
                None => assert!(
                    !is_carousel,
                    "{key} is a carousel and shot {index} ({}) has no control picture, \
                     so the control cannot bring the player there",
                    variant.path,
                ),
            }
        }
    }
}

/// Both halls offer the same two controls.
///
/// Whether the lift works is the difference between the two rows, and it is a
/// difference in the pan and the story line. The control is not part of that: the
/// player walks the corridor to reach the stairs either way, so a hall that grew its
/// own pair would be offering the same walk twice in two different sets of pictures.
#[test]
fn both_halls_offer_the_same_controls() {
    let working: Vec<_> = variant_previews(&room_def(p::F1_HALL, ActId::ActOne)).collect();
    let dead: Vec<_> = variant_previews(&room_def(p::F1_HALL_DEAD, ActId::ActOne)).collect();

    assert_eq!(
        working, dead,
        "the two halls offer different controls, so the same walk is drawn twice",
    );
    assert_eq!(
        working,
        vec![p::CAROUSEL_TO_ELEVATOR, p::CAROUSEL_TO_STAIRS],
        "the lift and the stairs controls are not the two that exist",
    );
}

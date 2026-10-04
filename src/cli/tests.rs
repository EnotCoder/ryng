//! Tests for `--rooms`.
//!
//! The risk in this is a query that resolves to the wrong room. A point-and-click
//! layout is easy to judge wrongly - you look at the wrong apartment and conclude
//! the corridor is fine - so a match that quietly prefers one of several
//! candidates would waste more time than a refused query does.
//!
//! Nothing here goes through [`parse`]: a query it cannot resolve prints and exits,
//! and a test process that exits takes the whole run with it. The deciding half,
//! [`resolve`], is what the tests drive.

use super::{FLAG, StartHere, by_name, by_number, listing, resolve, squash};
use crate::scenes::game::rooms::data::{act_of, key_of, rooms, rooms_with_act};

/// The room the flag should reach for a query that is meant to be unambiguous.
///
/// Written as a lookup rather than a literal so the test says what it means, and
/// so a failure names the room it reached instead.
fn starts_in(query: &str) -> StartHere {
    resolve(query).unwrap_or_else(|problem| panic!("`{query}` did not resolve: {problem}"))
}

/// The keys a refusal offers, pulled back out of its message.
///
/// The message is the only thing a user has to go on after a refusal, so testing
/// the message is testing the thing that has to be usable - not the formatting.
fn offered_keys(problem: &str) -> Vec<&str> {
    let mut keys = Vec::new();
    let mut rest = problem;
    while let Some(start) = rest.find('`') {
        rest = &rest[start + 1..];
        let Some(end) = rest.find('`') else { break };
        let quoted = &rest[..end];
        if quoted.ends_with(".png") {
            keys.push(quoted);
        }
        rest = &rest[end + 1..];
    }
    keys
}

// -- by number ---------------------------------------------------------------

/// Counting from 1, because that is what the listing prints. A `0` that quietly
/// meant "the first room" is the kind of off-by-one nobody notices.
#[test]
fn one_is_the_first_room() {
    let first = rooms().next().expect("rooms");
    assert_eq!(starts_in("1").room, key_of(first));
}

#[test]
fn the_numbers_count_along_the_route() {
    let keys: Vec<&str> = rooms().map(key_of).collect();
    for (index, key) in keys.iter().enumerate() {
        assert_eq!(
            starts_in(&(index + 1).to_string()).room,
            *key,
            "number {}",
            index + 1
        );
    }
}

/// The whole point of the feature: every room in the table is reachable by number,
/// with no gaps and nothing reached twice.
#[test]
fn every_room_is_reachable_by_its_number() {
    let expected: Vec<&str> = rooms().map(key_of).collect();
    let reached: Vec<&str> = (1..=expected.len())
        .map(|number| starts_in(&number.to_string()).room)
        .collect();
    assert_eq!(reached, expected);
}

/// A basement room has to report act 2, which is what makes `CurrentAct` right on
/// the first frame rather than after the first door - the concierge is the room
/// where the two disagree visibly.
#[test]
fn a_basement_room_starts_in_act_two() {
    let b_hall = starts_in("basement elevator hall");
    assert_eq!(b_hall.act, crate::acts::ActId::ActTwo);
}

/// The act has to come out right too, or `CurrentAct` disagrees with the room on
/// the very first frame.
#[test]
fn every_number_reaches_the_act_its_row_is_in() {
    for (index, (act, room)) in rooms_with_act().enumerate() {
        let here = starts_in(&(index + 1).to_string());
        assert_eq!(here.act, act, "number {} landed in the wrong act", index + 1);
        assert_eq!(
            act_of(key_of(room)),
            Some(here.act),
            "counting along the route and looking the act up disagree"
        );
    }
}

#[test]
fn zero_is_not_a_room() {
    let problem = by_number(0).expect_err("there is no room zero");
    assert!(
        problem.contains('0'),
        "the error does not mention what was asked for: {problem}"
    );
}

#[test]
fn a_number_past_the_end_is_refused() {
    let count = rooms().count();
    let problem = by_number(count + 1).expect_err("past the end");
    assert!(
        problem.contains(&(count + 1).to_string()),
        "the error does not say what was asked for: {problem}"
    );
    assert!(
        problem.contains(&count.to_string()),
        "the error does not say how many there are: {problem}"
    );
}

/// `checked_sub` is what stops room 0 wrapping round to the end of the table, so
/// this is the test that says it is there rather than merely correct.
#[test]
fn an_absurd_number_is_refused_rather_than_wrapping() {
    assert!(by_number(usize::MAX).is_err());
}

// -- by name -----------------------------------------------------------------

/// The picture name is the name that does not collide, so it is the one the flag
/// leans on.
#[test]
fn a_picture_name_resolves() {
    assert_eq!(starts_in("ap_3").room, "tex/rooms/floor_2/ap_3.png");
}

#[test]
fn a_picture_name_ignores_case() {
    let expected = starts_in("ap_3").room;
    for spelling in ["AP_3", "Ap_3", "aP_3"] {
        assert_eq!(starts_in(spelling).room, expected, "{spelling}");
    }
}

/// The same name written the way a person types it.
#[test]
fn punctuation_and_spacing_do_not_matter() {
    let expected = starts_in("ap_3").room;
    for spelling in ["ap 3", "ap-3", "ap.3", " ap_3 "] {
        assert_eq!(starts_in(spelling).room, expected, "{spelling}");
    }
}

#[test]
fn the_full_path_resolves() {
    assert_eq!(
        starts_in("tex/rooms/floor_2/ap_3.png").room,
        "tex/rooms/floor_2/ap_3.png"
    );
}

#[test]
fn the_path_without_its_extension_resolves() {
    assert_eq!(
        starts_in("tex/rooms/floor_2/ap_3").room,
        "tex/rooms/floor_2/ap_3.png"
    );
}

/// The title is what the game shows the player, so it has to work too - a layout
/// gets talked about by name long before anyone looks at the file.
#[test]
fn a_title_resolves() {
    assert_eq!(starts_in("apartment 3").room, "tex/rooms/floor_2/ap_3.png");
}

#[test]
fn a_title_with_punctuation_in_it_resolves() {
    assert_eq!(
        starts_in("basement elevator hall").room,
        "tex/rooms/basement/basement_with_elevator.png"
    );
}

/// The parked lobby has no way in, so `--rooms` is the only way to look at it.
/// Worth a test: the flag is the tool that makes unreachable content reachable.
#[test]
fn a_room_nothing_links_to_can_still_be_visited() {
    let lobby = starts_in("room with elevator floor my");
    assert_eq!(
        lobby.room,
        "tex/rooms/my_floor/room_with_elevator_floor_my.png"
    );
    assert_eq!(lobby.act, crate::acts::ActId::ActThree);
}

// -- ambiguity ---------------------------------------------------------------

/// The one thing this must never do: pick one of several. Two rooms are called
/// "Concierge" and the difference between them is the whole point of the room.
#[test]
fn a_name_several_rooms_share_is_refused() {
    let problem = by_name("concierge").expect_err("two rooms are called Concierge");
    for key in offered_keys(&problem) {
        assert!(key.contains("concierge"), "{key} is not a concierge: {problem}");
    }
    assert!(
        offered_keys(&problem).len() >= 2,
        "the lit and the dark concierge were not both offered: {problem}"
    );
}

/// Both halls share one stairs variant, so that title is genuinely two rooms.
/// The refusal is the correct answer, and it has to name both.
#[test]
fn a_shared_carousel_title_is_refused() {
    let problem = by_name("1st floor stairs").expect_err("both halls share this shot");
    let keys = offered_keys(&problem);
    assert_eq!(keys.len(), 2, "expected both halls, got {problem}");
    assert!(
        keys.iter().any(|key| key.contains("dont_work")),
        "the dead hall was not offered: {problem}"
    );
}

/// Refusing is only worth anything if the message is usable, so the keys it
/// offers have to resolve - each one back to a single room, with no new ambiguity.
#[test]
fn every_key_a_refusal_offers_resolves_unambiguously() {
    for query in ["concierge", "stairs", "1st floor stairs"] {
        let problem = by_name(query).expect_err("{query} is ambiguous");
        for key in offered_keys(&problem) {
            let here = resolve(key).unwrap_or_else(|again| {
                panic!("the refusal offered `{key}`, which then failed: {again}")
            });
            assert_eq!(here.room, key, "`{key}` resolved somewhere else");
        }
    }
}

/// The numbers are the way out of an ambiguity, so they have to be reachable -
/// which is the property that makes a number the canonical form.
#[test]
fn numbers_disambiguate_what_names_cannot() {
    let mut by_key = std::collections::HashMap::new();
    for (index, room) in rooms().enumerate() {
        by_key.insert(key_of(room), index + 1);
    }
    for key in by_key {
        let number = key.1;
        assert_eq!(starts_in(&number.to_string()).room, key.0);
    }
}

// -- refusing ----------------------------------------------------------------

#[test]
fn an_unknown_name_is_refused() {
    let problem = resolve("the moon").expect_err("no such room");
    assert!(
        problem.contains("the moon"),
        "the error does not quote the query: {problem}"
    );
}

#[test]
fn an_empty_query_is_refused() {
    assert!(resolve("").is_err());
    assert!(resolve("   ").is_err());
}

// -- the listing -------------------------------------------------------------

/// The listing is the answer to "what was that called?", so it has to name every
/// room and line the numbers up with what `--rooms` accepts.
#[test]
fn the_listing_names_every_room_and_the_right_numbers() {
    let listing = listing();
    let keys: Vec<&str> = rooms().map(key_of).collect();
    for (index, key) in keys.iter().enumerate() {
        assert!(listing.contains(key), "{key} is missing from the listing");
        assert!(
            listing.contains(&format!("{:>2}", index + 1)),
            "no number for row {index}"
        );
    }
    assert!(
        listing.contains(FLAG),
        "the listing does not say how to use the flag"
    );
}

/// The listing has to include the `--`, because omitting it produces a command
/// cargo rejects before the game ever starts.
#[test]
fn the_listing_shows_the_separator_cargo_needs() {
    assert!(
        listing().contains(&format!("cargo run -- {FLAG}")),
        "the printed command would be rejected by cargo"
    );
}

// -- the pieces --------------------------------------------------------------

/// Squashing is what makes the spellings match, so it is worth pinning: strip too
/// little and `ap-3` stops finding `ap_3`.
#[test]
fn squashing_reduces_a_name_to_its_letters() {
    assert_eq!(squash("Apartment 3"), "apartment3");
    assert_eq!(squash("ap_3"), "ap3");
    assert_eq!(squash("Basement - elevator hall"), "basementelevatorhall");
    assert_eq!(squash(""), "");
}

// -- the flag itself ---------------------------------------------------------

/// No flag, no override - the case that has to keep an ordinary `cargo run`
/// behaving exactly as it did.
#[test]
fn no_flag_means_no_override() {
    assert_eq!(super::parse(&[]), None);
}

/// Anything the game was not asked about is ignored, so an unrelated argument
/// cannot start the game in a room nobody named.
#[test]
fn arguments_that_are_not_the_flag_are_ignored() {
    for args in [
        vec!["--release".to_owned()],
        vec!["notes.txt".to_owned(), "--other".to_owned()],
    ] {
        assert_eq!(super::parse(&args), None, "{args:?} was taken as a request");
    }
}

/// `--rooms=7` as well as `--rooms 7`, because a shell that ate the space should
/// not silently start the game somewhere else.
#[test]
fn the_equals_spelling_is_the_same_flag() {
    let expected = starts_in("1").room;
    let here = super::parse(&[format!("{FLAG}=1")]).expect("a room");
    assert_eq!(here.room, expected);
}

#[test]
fn the_flag_resolves_through_parse() {
    let expected = starts_in("ap_3").room;
    let here = super::parse(&[FLAG.to_owned(), "ap_3".to_owned()]).expect("a room");
    assert_eq!(here.room, expected);
}
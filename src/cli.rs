//! `--rooms`: start the game in one room instead of at the beginning.
//!
//! ```sh
//! cargo run -- --rooms 7          # by number, counting along the route
//! cargo run -- --rooms ap_3       # by picture name
//! cargo run -- --rooms "apartment 3"
//! cargo run -- --rooms            # print every room, numbered
//! ```
//!
//! A point-and-click game is mostly walked through one room at a time, and the
//! walk is the slow part: sitting through the logo, finding the door that leads
//! where you want, and clicking it four times to get back. Naming the room skips
//! all of that, which is the difference between two seconds and a minute per
//! look at a layout.
//!
//! # Why it is a flag and not a menu
//!
//! Adding a room picker to the menu would mean a screen that exists only to
//! leave, reachable only when you already know the thing it does. The argument is
//! for whoever is changing the game, so it belongs on the command line where that
//! person already is.
//!
//! # Why the number is the way in
//!
//! Picture names are unstable: rename a picture and every invocation that named
//! it breaks. A number counted along `ACTS` moves when a room is added earlier in
//! the route, which is rarer and visible in the diff. So the number is the
//! canonical form and the names are the convenience, with the listing naming both
//! so there is never a guess.
//!
//! # The `--` is not optional
//!
//! `cargo run --rooms 7` is rejected by cargo before the game runs - it reads
//! `--rooms` as a cargo flag of its own. Everything after the first `--` is
//! passed straight through, so the separator is required. It is only one
//! character, but a rejected command teaches nothing, hence this paragraph.

use crate::acts::ActId;
use crate::scenes::game::rooms::data::{RoomDef, key_of, rooms_with_act};

#[cfg(test)]
mod tests;

/// The flag, as it appears on the command line.
const FLAG: &str = "--rooms";

/// The room to start in, and the act that owns it.
///
/// Both halves, because the act is not decoration: it is what makes the concierge
/// lit or dark, so a room reached by the flag has to arrive with the same act the
/// route would have given it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartHere {
    pub room: &'static str,
    pub act: ActId,
}

/// The override, or `None` for an ordinary run.
///
/// `None` is the common case and the only one that does anything but read the
/// room table, so a plain `cargo run` is unchanged: it parses two short strings
/// and stops.
pub fn from_args() -> Option<StartHere> {
    parse(&std::env::args().skip(1).collect::<Vec<_>>())
}

/// [`from_args`], over an argument list handed in rather than read from the
/// environment.
///
/// Split out because the deciding part is pure, and the part that is not - the
/// printing, and the exit - is the part worth not testing.
fn parse(args: &[String]) -> Option<StartHere> {
    let mut args = args.iter().map(String::as_str);
    while let Some(arg) = args.next() {
        let query = if arg == FLAG {
            // `--rooms` with nothing after it is the listing, not an error: asking
            // what the rooms are called is a reasonable thing to do by accident.
            match args.next() {
                Some(query) => query,
                None => {
                    println!("{}", listing());
                    return None;
                }
            }
        } else if let Some(query) = arg.strip_prefix(&format!("{FLAG}=")) {
            query
        } else {
            continue;
        };

        return match resolve(query) {
            Ok(here) => Some(here),
            Err(problem) => {
                eprintln!("{problem}\n");
                eprintln!("{}", listing());
                std::process::exit(2);
            }
        };
    }
    None
}

/// Find the room `query` names.
///
/// A number is the room's position in the table counting from 1, because that is
/// how the listing prints them. Anything else is matched against the names below.
///
/// A name is not unique and pretending otherwise would be worse than saying so:
/// two rooms are called "Concierge" and three are called "Stairs". So a query
/// that matches several is an error that prints the numbers, rather than a coin
/// flip that lands you in the wrong one half the time. The picture names have no
/// such problem - all 21 file names are distinct.
pub fn resolve(query: &str) -> Result<StartHere, String> {
    let query = query.trim();
    if query.is_empty() {
        return Err(format!("`{FLAG}` wants a room: a number, or a name."));
    }
    if let Ok(number) = query.parse::<usize>() {
        return by_number(number);
    }
    by_name(query)
}

/// The room at `number`, counting from 1 along [`rooms_with_act`].
fn by_number(number: usize) -> Result<StartHere, String> {
    let rows: Vec<(ActId, &'static RoomDef)> = rooms_with_act().collect();
    match number.checked_sub(1).and_then(|index| rows.get(index)) {
        Some((act, room)) => Ok(StartHere {
            room: key_of(room),
            act: *act,
        }),
        None => Err(format!(
            "there {} {} rooms, so `{number}` is not one of them.",
            if rows.len() == 1 { "is" } else { "are" },
            rows.len()
        )),
    }
}

/// The one room `query` names, or an error saying which ones it might mean.
fn by_name(query: &str) -> Result<StartHere, String> {
    let wanted = squash(query);
    let hits: Vec<(ActId, &'static RoomDef)> = rooms_with_act()
        .filter(|(_, room)| names_of(room).any(|name| squash(name) == wanted))
        .collect();

    match hits.as_slice() {
        [] => Err(format!("no room is called `{query}`.")),
        [(act, room)] => Ok(StartHere {
            room: key_of(room),
            act: *act,
        }),
        // Named with their numbers, because that is what resolves the ambiguity.
        _ => Err(format!(
            "`{query}` could be any of {} rooms: {}.",
            hits.len(),
            hits.iter()
                .map(|(_, room)| format!("`{}`", key_of(room)))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Every name a room answers to.
///
/// The path itself, the path without its extension, the picture's file name, and
/// the title of each variant. The path forms are three spellings of one thing and
/// cost nothing to accept; the titles are there because they are what the game
/// shows the player, so `basement hall` finds the room the listing calls
/// "Basement - elevator hall".
fn names_of(room: &RoomDef) -> impl Iterator<Item = &str> {
    let path = key_of(room);
    let stem = path.rsplit('/').next().unwrap_or(path);
    let stem = stem.strip_suffix(".png").unwrap_or(stem);
    room.variants
        .iter()
        .map(|variant| variant.title)
        .chain([path, &path[..path.len() - ".png".len()], stem])
}

/// Reduce a name to something worth comparing.
///
/// Letters and digits only, lowercased: `ap_3`, `ap 3`, `ap-3` and `AP_3` are one
/// name written four ways, and a title with punctuation in it should not need
/// quoting at the shell to be matched.
fn squash(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// The rooms, numbered the way `--rooms` counts them.
///
/// Printed on a bare `--rooms` and on any query that does not resolve, because
/// the answer to "it said no room is called X" is a list of what it would have
/// accepted.
pub fn listing() -> String {
    let mut out = String::from("rooms:\n");
    for (index, (act, room)) in rooms_with_act().enumerate() {
        out.push_str(&format!(
            "  {:>2}  act {}  {:<32}  {}\n",
            index + 1,
            number_of(act),
            room.variants[0].title,
            key_of(room),
        ));
    }
    out.push_str(&format!(
        "\nstart in one of them:  cargo run -- {FLAG} <number or name>\n"
    ));
    out
}

/// 1, 2, 3 for the listing, where the act has to be scannable at a glance.
///
/// An exhaustive `match` rather than a cast from the discriminant, so a fourth
/// act is a compile error here instead of a row silently numbered 3.
fn number_of(act: ActId) -> usize {
    match act {
        ActId::ActOne => 1,
        ActId::ActTwo => 2,
        ActId::ActThree => 3,
    }
}
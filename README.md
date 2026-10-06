# ryng

A point-and-click narrative game built with [Bevy](https://bevyengine.org) 0.19 and
Rust 2024. You walk a tired clerk home from work, the elevator drops you into the
basement, and the way back up is not the way you came.

Everything is a still picture of a room and a list of rectangles you can click. No
character controllers, no physics — the whole game is rooms, hotspots, an
inventory and a fade between them.

## How to run

Needs [Rust](https://rustup.rs) and the system dependencies for Bevy (see the
[Bevy setup guide](https://bevyengine.org/learn/quick-start/getting-started/setup/)).

```sh
cargo run          # play
cargo test         # unit tests
cargo clippy --all-targets
```

### Starting in one room

```sh
cargo run -- --rooms 7              # by number, counting along the route
cargo run -- --rooms ap_3           # by picture name
cargo run -- --rooms "apartment 3"
cargo run -- --rooms                # print every room, numbered
```

Skips the intro and the menu and opens in that room, which is the difference
between two seconds and a minute per look at a layout. The one-second splash
stays: it covers the first frame of asset loading.

The number is the canonical form and the names are the convenience, because
picture names break when a picture is renamed and a number only moves when a room
is added earlier in the route. Names are matched against the path, the file name
and the title, ignoring case and punctuation — so `ap_3`, `ap 3`, `AP_3` and
`ap-3` are one room.

**A name several rooms share is refused, not guessed.** Two rooms are called
`Concierge` and three are called `Stairs`, and the difference between the two
concierges is the whole point of that room, so the error lists the pictures it
could have meant and those resolve unambiguously.

The `--` is required: `cargo run --rooms 7` is rejected by cargo before the game
starts, because it reads `--rooms` as a flag of its own.

It is also the only way to reach the parked lobby on floor 2, which nothing in
the game links to yet.

## What's in it

- **Four states**, wired with Bevy's `States`: `Loading` -> `Intro` -> `Menu` -> `Game`
- **Intro**: the logo falls in, lands, and drops out on a fixed 4.8s curve
- **Menu** with `Settings`, `Play` and `Quit`
- **Settings** panel: a volume slider, generic over the resource it edits, so a new
  setting is a type plus one line
- **Rooms** declared as data, not code: 21 rows split one file per act, each with a
  picture, a title, a story line, transition sound, music and its hotspots
- **Characters** with dialogue: click the granny in the concierge and she warns you
  about the elevator; any click puts the line away, the next one starts the next line
- **Four kinds of room**: interactive (`room!`), a beat that plays and moves on by
  itself (`beat!` / `chapter!`), a carousel of two or more pictures the player
  flips through (`carousel!`), and a repeated floor-2 apartment (`apartment!`)
- **Hotspots**: `hop!` an ordinary door, `gated!` one that spends a single item,
  `locked!` one that needs a set of items carried but not spent, `take!` / `drop!`
  for items on the floor
- **Hover outline**: pointing at a hotspot draws a white frame around it, pulsing
  gently so it reads as a highlight rather than as part of the picture. Four bars
  rather than a texture, because every row in the table has its own size
- **Inventory** of four slots with an active slot; doors that need an item only
  open when it is the one selected
- **Six items**, each with one icon path and one slot texture table entry
- **Three acts**: *The Curse*, *The Descent*, *The Escape*. Act 3 is stubbed
- Per-room music and transition sounds, cross-faded through a 0.35s fade overlay
- **Camera** `FixedVertical` at `DESIGN_HEIGHT = 720`, so a room picture maps to
  the frame one to one and every UI constant can be authored against 720
- **UI follows the window**: constants stay authored against 720 and are scaled by
  `UiScale` at runtime. A node carrying `ScaledNode` and a text carrying
  `ScaledFont` have their design-space numbers replayed whenever the window changes;
  every field is an `Option`, and a `None` is left alone
- **Android** target under `mobile/`, built with `cargo-apk`

## Controls

| Input        | Action                                             |
| ------------ | -------------------------------------------------- |
| Left mouse   | Press / hover / release to click                  |
| `Back`       | Return to the menu from a room                     |
| `<` / `>`     | Flip between the pictures of a carousel room       |
| Character     | Talk; click anywhere to put the line away           |
| Inventory    | Click a slot to make it the active one             |
| `Quit`       | Exit from the menu                                 |

There is no keyboard input in the game at all. The arrow keys you may reach for do
nothing, and that is deliberate — the game is played entirely with the mouse.

## Project structure

```
src/
├── main.rs          # Thin entry point, calls into the lib
├── lib.rs           # App wiring, camera, UiScale, DESIGN_HEIGHT/FRAME_HALF
├── state.rs         # GameState: Loading / Intro / Menu / Game
├── acts.rs          # Item, Inventory, ActId, Act
├── buttons/         # Button spawning, hover/press colour states, click reader
├── cli.rs           # `--rooms`: start in one room, skip the intro and the menu
└── scenes/
    ├── loading.rs   # Splash and the asset preload overlay
    ├── intro/       # Logo fall curve
    ├── menu/        # Menu buttons
    ├── settings/    # Slider widget and the volume resource
    ├── fade.rs      # RoomFade state machine, drives transitions and act changes
    ├── sound.rs     # Music / TransitionSound tables
    └── game/
        ├── mod.rs           # GamePlugin
        ├── systems.rs       # Hotspot clicks, carousel, room breathing, icon blink
        ├── ui.rs            # Back button, carousel arrows, title, dialogue box
        ├── items.rs         # World items, take/drop rules, item sprite syncing
        ├── inventory/       # Inventory UI and active slot
        ├── npc/             # Characters: sprites, click targets, dialogue
        └── rooms/
            ├── components.rs  # Room, Hotspot, HotspotDef, RoomVariant
            ├── spawn.rs       # Room and hotspot spawning
            └── data/
                ├── p.rs        # Asset paths, one constant per picture
                ├── builders.rs # The room!/hop!/take! macro family
                ├── data.rs     # Lookup by path, preload list, rooms()
                └── table/
                    ├── mod.rs      # ACTS: the acts, in play order, plus UNKNOWN
                    ├── act_one.rs  # Street, concierge, hall, the lift
                    ├── act_two.rs  # The basement and the way back up
                    └── act_three.rs# Floor 2 and the three apartments

assets/
├── tex/
│   ├── rooms/       # floor_1, floor_2, my_floor, basement, stairs
│   └── ui/          # buttons, inventory slots and icons, room title plate
└── sounds/          # Music loops and transition one-shots

mobile/              # Gradle project for the Android build
```

The tree above is written by hand, which means it was true on the day it was typed.
`helper.py` is the same map, checked against the disk:

```sh
python3 helper.py                    # lines per file and folder, code vs tests
python3 helper.py about rooms/data   # what a module is for, who uses it
python3 helper.py doctor             # known problems and unfinished markers
python3 helper.py help               # every command
```

`about` takes a fragment, not a full path, and will name every module it could mean
rather than picking one — `about tests` finds nine `tests.rs` in nine directories,
and guessing which one you meant would be worse than asking. It also reports what a
module exports and which files depend on it, read from the `use`, `mod` and inline
`super::` references in the source.

The descriptions are a table in the script rather than scraped from the modules,
because fewer than half of them carry a `//!` header and a description for 22 files
is not much of a map. `python3 helper.py docs` fails when a module has no entry, an
entry names a file that is gone, or an entry is left empty — that is the check to
run after adding or renaming a module.

Standard library only. It is not in `Cargo.toml` and never will be, so it cannot
drift from the Android build.

## The room table

Adding a room is one row, not a function. `src/scenes/game/rooms/data.rs` looks a
room up by the path of its first variant, so that path is the room's key; an asset
path is a constant in `p.rs`. The preload list is derived from the same table, so
it cannot drift out of sync with the rooms that exist.

```rust
room!(
    p::F2_CORRIDOR,
    "Floor 2 - Corridor",
    "",
    TransitionSound::NextRoom,
    Music::Indoors,
    &[
        hop!(p::F2_HALL, 0.0, -270.0, Vec2::new(200.0, 100.0)),
        hop!(p::AP_3, -500.0, 0.0, Vec2::new(200.0, 500.0)),
        locked!(p::STAIRS_1, 0.0, 0.0, Vec2::new(400.0, 400.0), &Item::DOOR_TOOLS)
    ]
),
```

The builders are macros rather than `const fn`s on purpose: a `&[..]` can only be
promoted to `'static` where the borrow is written syntactically, so expanding the
macro at the `static` is what puts every literal into the constant initializer.

`locked!` takes a slice so a set can be named at the use site —
`Item::DOOR_TOOLS` is the three tools the black door on floor 2 wants, and none of
them is spent, so the player can still walk back through afterwards.

### Reading a `hop!` line

`hop!(p::F2_CORRIDOR, 555.0, 5.0, Vec2::new(150.0, 610.0))` is a door leading to
the floor 2 corridor, 555px from the left edge of the picture and 5px down from the
top, at 150x610 instead of the default 200x300.

Coordinates are `x` from the left edge of the picture and `y` down from the top, in
pixels of the 1280x720 art, because the camera is `FixedVertical` at 720. The
world coordinates the engine works in are the same numbers with `y` flipped —
positive `y` is up, so the table's `y = -210` is 210px *down* the picture.

Two things decide where a line goes:

- **The target names the direction, not the room.** A line pointing at `ap_3.png`
  from inside apartment 3 belongs in `apartment!(p::AP_3, ...)`. The same picture
  as a target from the corridor belongs in the corridor's row.
- **Which numbers you paste matters.** The builders take arguments positionally.
  `apartment!` is `path, title, exit, exit_x, exit_y, exit_size, teddy_x, teddy_y,
  tool, tool_x, tool_y` — so `exit_x, exit_y, exit_size` are three consecutive
  slots, and a size pasted into the teddy's slot compiles, runs, and quietly
  moves the teddy instead of resizing the door. `cargo test` checks every
  apartment's teddy spot is inside the frame and will name the room for you.

### One file per act

The rows are split across `table/act_one.rs`, `act_two.rs` and `act_three.rs`, and
`table/mod.rs` lists them in `ACTS` as the single statement of play order. That
order is load-bearing: `data::tests` pins the whole route as a literal vector, so
moving a row between files has to be a deliberate edit to that test rather than a
reordering that happens to play identically.

A room belongs to the act that is current *while the player stands in it*, and the
act changes on entry to the room carrying `next_act` — so that room is the first
row of the next act's file:

| Act | Opens at | Carries `next_act` |
| --- | --- | --- |
| 1 The Curse | the street outside home | — |
| 2 The Descent | the basement hall | `ActTwo` |
| 3 The Escape | the floor 2 hall | `ActThree` |

Two placements that look wrong until you trace the route, both commented in place:
the elevator beat is act 1 (the fall ends the ride the player started in act 1),
and `STAIRS_1`/`STAIRS_2` are act 2 (the player reaches them from the basement, and
the floor 2 black door sends them back down through them).

Adding an act is a new file with a `ROOMS`, one line in `ACTS`, and whatever
`next_act` the last room of the previous act should carry. Nothing outside
`table/` changes: everything else reads the rooms through `data::rooms()`, which
flattens `ACTS`.

## Characters

An NPC is a sprite, a clickable rectangle, and a list of lines. Adding one is a
row in `src/scenes/game/npc/data.rs`:

```rust
pub const GRANNY: NpcDef = NpcDef {
    id: NpcId::Granny,
    texture: "tex/npc/granny.png",
    room: p::F1_CONCIERGE,
    pos: Vec2::new(170.0, 105.0),
    size: Vec2::new(120.0, 200.0),
    hit: Vec2::new(130.0, 215.0),
    lines: &["...", "..."],
};
```

All in-game text is English. A test walks the dialogue table and fails on a
non-ASCII line, so a stray translation does not ship unread.

Click her and the line goes into a box on screen. While it is there, **any** click
puts it away — clicking the wall, clicking a door, clicking her again. There is no
button behind it: the system reads every click in the app, including the ones that
landed on nothing. From empty, clicking her starts the next thing she has to say.
Once her lines run out the last one repeats, because a character who goes silent
reads as a bug rather than as an ending. The count is kept per character rather
than per room, so leaving and coming back does not reset it.

A dismiss does not count as a conversation, so clicking twice to move on does not
skip what she would have said. And because `MessageReader` cannot consume a click,
dismissing never blocks the game: a click on a door both puts the line away and
walks the player through, rather than trapping them until the timer expires.

Characters are deliberately not hotspots. A hotspot moves the player or changes
the inventory; clicking a character only produces speech, so routing it through
`HotspotAction` would mean either adding a variant that cannot move the player or
special-casing doors out of the existing one.

Three placement rules the tests enforce:

- **She must not cover the way out.** The concierge's exit is a 200x300 hotspot
  across the middle of the room, and her target is drawn above the hotspot layer,
  so overlap means clicking the door talks to her instead and the player cannot
  leave. That is a soft lock. The recess in the desk runs to about `x = 240` while
  the door stops at `x = 100`, which is the room she has to stand in.
- **The click target is above the hotspot layer.** Picking hands a click to the
  topmost entity at that point, so a click aimed at her has to land on her.
- **`room` names one row, not a place.** The concierge has two rows: lit and dark.
  She is in the lit one, because the dark row says nobody is on duty.


## License

MIT

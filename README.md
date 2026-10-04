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

Two optional debug features, each of which is compiled out entirely without its
flag: `--features hotspot-editor` (below) and `--features inspector`.

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
- **Inventory** of four slots with an active slot; doors that need an item only
  open when it is the one selected
- **Six items**, each with one icon path and one slot texture table entry
- **Three acts**: *The Curse*, *The Descent*, *The Escape*. Act 3 is stubbed
- Per-room music and transition sounds, cross-faded through a 0.35s fade overlay
- **Camera** `FixedVertical` at `DESIGN_HEIGHT = 720`, so a room picture maps to
  the frame one to one and every UI constant can be authored against 720
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

There is no keyboard input in the game itself. The arrow keys you may reach for do
nothing, and that is deliberate — the only keyboard bindings in the codebase belong
to the hotspot editor below.

## Project structure

```
src/
├── main.rs          # Thin entry point, calls into the lib
├── lib.rs           # App wiring, camera, UiScale, DESIGN_HEIGHT/FRAME_HALF
├── state.rs         # GameState: Loading / Intro / Menu / Game
├── acts.rs          # Item, Inventory, ActId, Act
├── buttons/         # Button spawning, hover/press colour states, click reader
└── scenes/
    ├── loading.rs   # Splash and the asset preload overlay
    ├── intro/       # Logo fall curve
    ├── menu/        # Menu buttons
    ├── settings/    # Slider widget and the volume resource
    ├── fade.rs      # RoomFade state machine, drives transitions and act changes
    ├── sound.rs     # Music / TransitionSound tables
    └── game/
        ├── mod.rs           # GamePlugin, gameplay_active()
        ├── systems.rs       # Hotspot clicks, carousel, room breathing, icon blink
        ├── ui.rs            # Back button, carousel arrows, title, dialogue box
        ├── items.rs         # World items, take/drop rules, item sprite syncing
        ├── inventory/       # Inventory UI and active slot
        ├── npc/             # Characters: sprites, click targets, dialogue
        ├── hotspot_edit.rs  # The editor, only with the feature on
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

### One file per act

The rows are split across `table/act_one.rs`, `act_two.rs` and `act_three.rs`, and
`table/mod.rs` lists them in `ACTS` as the single statement of play order. That
order is load-bearing: the hotspot editor steps through rooms with `[` and `]`
and prints "room 7/21", so moving a row between files renumbers the editor even
though the game plays identically. `data::tests` pins the whole route to catch
that.

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

## Inspector

Almost everything worth checking while playing is the state of one entity: which
room is loaded, which hotspots are currently live, what the inventory holds, what
`RoomFade.phase` is doing mid-transition. That is what the ECS inspector is for.

```sh
cargo run --features inspector
```

It draws a floating, draggable window over the game with the entity hierarchy,
the components on whatever is selected, and the lists of resources and assets -
click a resource to see its contents.

Two things about it are worth knowing before you trust it:

- **It is not a second window.** It is an egui overlay inside the game window, so
  it shows up in screenshots of the game and it can cover a corner of the picture.
  bevy_egui hands its primary context to the first camera that appears, so nothing
  has to be attached to `spawn_camera` by hand.
- **The panel has to be told it goes on top.** `bevy-inspector-egui` depends on
  `bevy_egui` with `default-features = false`, which leaves out bevy_egui's
  `bevy_ui` feature - and `EguiPlugin::ui_render_order` is `#[cfg(feature =
  "bevy_ui")]`, so with it off the plugin has no opinion about where it is drawn
  and the game's own Bevy UI renders straight over the panel. The fix is the
  `bevy_egui` line in `Cargo.toml`, which names that one feature; Cargo unifies
  features across the graph, and `EguiAboveBevyUi` is then the default.
- **It does not eat your clicks.** `EguiPlugin` leaves
  `enable_absorb_bevy_input_system` off, so it never clears `MouseButtonInput`
  and a click that lands on the panel still reaches `Pointer<Click>` underneath.
  Clicking a door walks the player through *and* selects the hotspot.

And one thing about what it can show:

- **Component names, but not component values.** Nothing in this game derives
  `Reflect`, and bevy_inspector_egui needs it to read a component's contents. So
  the tree lists `Room`, `HotspotDef`, `Inventory`, `RoomFade` by name, and
  opening one says *No access to component ...*. The entities are unnamed too, so
  a hotspot is just `Entity (512v0)` - see `utils::guess_entity_name`. Deriving
  `Reflect` and registering the types would make the inspector actually live,
  which is the difference between reading it and using it.

Behind the feature with the editor below, for the same reason and with the same
guarantee: a plain `cargo run` does not link the crate or its egui stack, so none
of it can reach a release build. It is also desktop-only, so it is not part of
the Android build.

## Hotspot editor

Coordinates are `x` from the left edge and `y` down from the top, measured off the
room picture. Getting them by hand is tedious, so there is an overlay:

```sh
cargo run --features hotspot-editor
```

It draws a faint rectangle over every hotspot in the room, brighter over the
selected one, with a readout along the bottom.

| Input      | Action                                             |
| ---------- | -------------------------------------------------- |
| Left click | Select a hotspot                                   |
| Arrows     | Move by 5px, 1px with Shift held                   |
| `Q` / `E`  | Shrink / grow by 5px on both axes                  |
| `A` / `D`  | Width only                                         |
| `W` / `S`  | Height only                                        |
| `C`        | Copy the definition line for the current placement  |
| `Esc`      | Drop the selection                                 |
| `[` / `]`  | Previous / next room in the table                  |
| `G`        | Jump to the first room of the current act          |

The per-axis keys are the ones you want: a door frame is wide and short, and
reaching that from the default rectangle by growing both sides in step means
walking the width back down five pixels at a time.

The readout names the room, the act, the hotspot index and the definition line,
and lists any other hotspot the selection overlaps — an overlap means the top one
swallows the clicks meant for the one underneath, which is better to find while
placing things than after.

It never writes to the source. It shows you the line and puts it on the
clipboard, and you paste it: a tool that edits your code while you hold the arrow
keys is how you end up with a diff you cannot explain.

The overlay is behind a Cargo feature, so a plain `cargo run` compiles the module
out entirely and it cannot reach a release build. With the feature on, the editor
always wins the pointer, so clicks select instead of walking the player through
the door.

### Reading the line it gives you

`hop!(tex/rooms/floor_2/room_2.png, 555.0, 5.0, Vec2::new(150.0, 610.0))` means a
door leading to the floor 2 corridor, 15px right of where it was and 5px up, at
150x610 instead of the default 200x300. The size is only printed when it differs
from the default, so a line without one is not missing anything.

Two things decide where the line goes:

- **The target names the direction, not the room.** This one points *out of*
  apartment 3, so it belongs in `apartment!(p::AP_3, ...)` in `table.rs`. A line
  pointing at `ap_3.png` would instead belong to the corridor's row.
- **Which numbers you paste matters.** The builders take arguments positionally.
  `apartment!` is `path, title, exit, exit_x, exit_y, exit_size, teddy_x, teddy_y,
  tool, tool_x, tool_y` — so `exit_x, exit_y, exit_size` are three consecutive
  slots, and a size pasted into the teddy's slot compiles, runs, and quietly
  moves the teddy instead of resizing the door. `cargo test` checks every
  apartment's teddy spot is inside the frame and will name the room for you.

Coordinates are `x` from the left edge of the picture and `y` from the top, in
pixels of the 1280x720 art, because the camera is `FixedVertical` at 720. The
world coordinates the engine works in are the same numbers with `y` flipped —
positive `y` is up, so the table's `y = -210` is 210px *down* the picture. The
editor prints picture convention, since that is what you measure off the art.

## License

MIT

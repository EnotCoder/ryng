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

## What's in it

- **Four states**, wired with Bevy's `States`: `Loading` -> `Intro` -> `Menu` -> `Game`
- **Intro**: the logo falls in, lands, and drops out on a fixed 4.8s curve
- **Menu** with `Settings`, `Play` and `Quit`
- **Settings** panel: volume slider and a vignette toggle, both generic over the
  resource they edit, so a new setting is a type plus one line
- **Rooms** declared as data, not code: 21 rows in one table, each with a picture,
  a title, a story line, transition sound, music and its hotspots
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
    ├── settings/    # Slider + checkbox widgets, volume and vignette resources
    ├── fade.rs      # RoomFade state machine, drives transitions and act changes
    ├── sound.rs     # Music / TransitionSound tables
    └── game/
        ├── mod.rs           # GamePlugin, gameplay_active()
        ├── systems.rs       # Hotspot clicks, carousel, room breathing, icon blink
        ├── ui.rs            # Back button, carousel arrows, title/story, preload
        ├── items.rs         # World items, take/drop rules, item sprite syncing
        ├── inventory/       # Inventory UI and active slot
        ├── hotspot_edit.rs  # The editor, only with the feature on
        └── rooms/
            ├── components.rs  # Room, Hotspot, HotspotDef, RoomVariant
            ├── spawn.rs       # Room and hotspot spawning
            └── data/
                ├── p.rs        # Asset paths, one constant per picture
                ├── builders.rs # The room!/hop!/take! macro family
                ├── table.rs    # The rows
                └── data.rs     # Lookup by path, preload list

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

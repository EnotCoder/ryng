# ryng

A reaction-testing game prototype built with [Bevy](https://bevyengine.org) 0.19 and Rust 2024.

## Screenshot / Scene

The game takes place in a room rendered by `assets/room.png`. Click the target as fast as you can when it appears — the core reaction loop is the next step.

## Features

- **Main menu** with `Settings`, `Play`, and `Quit` buttons
- Button hover/press **color animations** (release-to-click, same as Godot-style UI)
- **Scene transitions** powered by Bevy `States`:
  - `Menu` -> `Game` via `Play`
  - `Game` -> `Menu` via `Back`
  - `Quit` exits the application
- Game scene with a scalable **sprite background** (`Transform.scale`, Godot-style)
- Scene-scoped entities auto-despawned on exit (`DespawnOnExit`)

## Controls

| Input          | Action                    |
| -------------- | ------------------------- |
| Left mouse     | Press / hover / release to click |
| `Quit` button  | Exit the application      |

## Project structure

```
src/
├── main.rs      # App setup, camera, UI spawning, state wiring
├── button.rs    # Button rendering + interaction systems (colors, clicks)
└── state.rs     # GameState (Menu / Game)
assets/
└── room.png     # Game scene background
```

## How to run

Make sure you have [Rust](https://rustup.rs) and system dependencies for Bevy installed
(see the [Bevy setup guide](https://bevyengine.org/learn/quick-start/getting-started/setup/)).

```sh
cargo run
```

## Hotspot editor

Rooms and their interactive spots are declared in a table in
`src/scenes/game/rooms/data.rs`, as one line per hotspot (`hop!`, `take!`,
`drop!`, `locked!`, `gated!`). Getting those coordinates right by hand off the
room art is tedious, so there is an overlay for it:

```sh
cargo run --features hotspot-editor
```

It draws a faint rectangle over every hotspot in the current room, brighter over
the selected one, with a readout along the bottom. It never writes to the source:
it shows you the definition line and puts it on the clipboard, and you paste it.

| Input              | Action                                            |
| ------------------ | ------------------------------------------------- |
| Left click         | Select a hotspot                                  |
| Arrow keys         | Move it by 5px, 1px with Shift held               |
| `Q` / `E`          | Shrink / grow by 5px                              |
| `C`                | Copy the definition line for the current placement |
| `Esc`              | Drop the selection                                |
| `[` / `]`          | Previous / next room in the table                 |
| `G`                | Jump to the first room of the current act         |

The readout names the room and act, the hotspot index, the definition line, and
any other hotspot the selection overlaps — an overlap means the top one swallows
the clicks meant for the one underneath, which is worth knowing while you place
things rather than after.

Room art is 1280x720 and the camera is `FixedVertical`, so the numbers in the
table map to the picture one to one: `x` is measured from the left edge and `y`
down from the top.

The overlay is behind a Cargo feature, so `cargo run` compiles it out entirely
and it cannot reach a release build. With the feature on, the editor always wins
the pointer, so clicks select instead of walking the player through the door.

## Roadmap

- Reaction round loop: `Idle -> Waiting -> Ready`
- Reaction-time measurement and last/best time display
- Settings scene

## License

MIT
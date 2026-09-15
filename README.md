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

## Roadmap

- Reaction round loop: `Idle -> Waiting -> Ready`
- Reaction-time measurement and last/best time display
- Settings scene

## License

MIT
## shinra-examples

Sample game data for [`shinra-engine`](../shinra-engine/). This repo holds no
Rust code — games are pure data (`scene.ron`) plus shared mesh, image, and tile
assets.

Clone both repos as siblings:

```
shinra-engine/
├── shinra-engine/
└── shinra-examples/      ← you are here
```

## Layout

```
shinra-examples/
├── assets/
│   ├── games/
│   │   ├── game1/   scene.ron — bunny scene
│   │   ├── game2/   scene.ron — teapot scene
│   │   ├── game3/   scene.ron — dino-run mini game (sprites + run mode)
│   │   └── game4/   scene.ron — galgame dialogue (Space advances text)
│   ├── obj/         bunny.obj, teapot.obj, quad.obj, quad_xy.obj
│   ├── images/      2x2_grid.png sprite sheet (dino, tree, cloud, bird)
│   ├── scenes/      legacy single-scene `.scn.ron` files
│   └── tilesets/    `.tres.ron` tilesets
```

A "game" under `assets/games/<name>/` is one RON file:

- `scene.ron` — a `scene::Scene`: nodes with transforms, mesh refs
  (`assets/obj/...`), sprites (`assets/images/...` sheet + grid cell),
  tilemaps, behavior components, and an optional embedded camera.

The TUI scans this directory at startup, loads the first game, and **`n`** in
the viewport cycles to the next.

## Run the TUI

Run the engine from this directory so relative asset paths resolve against the
examples project:

```bash
cargo run -p tui --manifest-path ../shinra-engine/Cargo.toml
```

To keep it in a dedicated tmux session:

```bash
tmux new-session -s sh-run -c "$PWD" \
  'cargo run -p tui --manifest-path ../shinra-engine/Cargo.toml'
```

| Key | Action |
|---|---|
| **r** | toggle running mode |
| **Space** | next dialogue line, or jump while running |
| **n** | cycle to the next game |
| **q** / **Esc** | quit |

## Adding a new game

```bash
mkdir -p assets/games/game3
cp assets/games/game1/scene.ron assets/games/game3/
# edit the new scene.ron, then restart the TUI to rescan game directories
```

## shinra-examples

Sample game data + a docker-compose for the
[`shinra-engine-core`](../shinra-engine-core/) editor-server. This repo holds
no Rust code — games are pure data (`scene.ron` + `tscn.ron`) and shared
mesh/tile assets.

Clone both repos as siblings:

```
shinra-engine/
├── shinra-engine-core/
└── shinra-examples/      ← you are here
```

## Layout

```
shinra-examples/
├── assets/
│   ├── games/
│   │   ├── game1/   scene.ron + tscn.ron — bunny scene
│   │   └── game2/   scene.ron + tscn.ron — teapot scene
│   ├── bunny.obj, teapot.obj, quad.obj
│   ├── scenes/      legacy single-scene `.scn.ron` files
│   └── tilesets/    `.tres.ron` tilesets
└── docker-compose.yml   launches the editor-server with this folder as /game
```

A "game" under `assets/games/<name>/` is just two RON files:

- `scene.ron` — a `scene::Scene` (nodes, transforms, mesh refs, tilemaps)
- `tscn.ron` — a `scene::Camera` (eye, target, up, perspective/orthographic)

The editor-server scans this directory at startup, loads the first one, and
**`n`** in the viewport cycles to the next.

## Run the editor-server (VS Code viewport)

The editor-server renders the active game as an H.264 stream consumed by the
Shinra VS Code extension. It runs in Docker so no local Rust toolchain or GPU
is required on this side.

```bash
# One-time: build the engine image (from the sibling core repo)
cd ../shinra-engine-core
docker build -t shinra-editor-server .

# Each session: launch the server from this folder
cd -
docker compose up
```

The compose file bind-mounts `.` as `/game`, mounts `~/.cargo` so the dev
image reuses the host's crates cache, and binds HTTP `:5812` + WS `:5813`.

The container runs as UID 1000 so files saved back stay owned by you, not
root. If your UID isn't 1000, edit the `user:` line in `docker-compose.yml`.

Open `shinra-examples/` in VS Code, run **Shinra: Open Viewport** from the
command palette — the live render appears in a webview.

| Key | Action |
|---|---|
| Arrow keys | move node 0 in screen-X / screen-Y |
| **n** | cycle to next game |

## Adding a new game

```bash
mkdir -p assets/games/game3
cp assets/games/game1/{scene.ron,tscn.ron} assets/games/game3/
# edit the new scene.ron + tscn.ron
docker compose restart        # editor-server picks up the new directory
```

No build step — `assets/games/*` is rescanned every container start.

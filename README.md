## shinra-examples

Sample games for the [`shinra-engine-core`](../shinra-engine-core/) engine.
Each game is a `cdylib` that the core's `runner` dlopens at runtime.

Clone both repos as siblings:

```
shinra-engine/
├── shinra-engine-core/
└── shinra-examples/      ← you are here
```

Games' `Cargo.toml` reach the engine's `abi/` and `scene/` crates via
relative path deps (`../../../shinra-engine-core/{abi,scene}`), so the
sibling layout matters.

## Workspace layout

```
shinra-examples/
├── games/
│   ├── game1/   bunny   (.hom)
│   ├── game2/   teapot  (.hom)
│   └── game3/   .hom example using `scene`
└── assets/      bunny.obj, teapot.obj, ...
```

## Build

```bash
cargo build                       # builds all games → target/debug/libgame*.so
```

Then run the engine from the core repo:

```bash
cd ../shinra-engine-core
cargo run -p runner               # cycles libgame*.so files it can find
```

The runner scans `target/debug/` of its own repo. Either symlink the
example `.so` files in, or copy them:

```bash
cp target/debug/libgame*.so ../shinra-engine-core/target/debug/
```

## Writing a new game

See [`shinra-engine-core/README.md`](../shinra-engine-core/README.md) for the
full mini-game tutorial (mesh loading, `.hom` DSL, the FFI contract). The
short version:

```bash
cp -r games/game1 games/game4
sed -i 's/name = "game1"/name = "game4"/' games/game4/Cargo.toml
# add games/game4 to this workspace's Cargo.toml [workspace.members]
```

Edit `games/game4/src/lib.rs` (`MESH_PATHS`) and `games/game4/src/main.hom`
for the gameplay.

## `.hom` DSL prerequisite

`game1`, `game2`, `game3` are written in Homun DSL and need the `homunc`
compiler at `.tmp/homunc`:

```bash
mkdir -p .tmp
curl -L https://github.com/homun-lang/homun/releases/latest/download/homunc-linux-x86_64 \
  -o .tmp/homunc
chmod +x .tmp/homunc
```

Each game's `build.rs` finds it automatically.

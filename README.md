# shinra-examples

Games for the [Shinra Engine](../shinra-engine).

**All game logic lives here.** Every rule, every behaviour and every number
that belongs to a game is a file in this repository. The engine holds none of
it: it provides general essential tools — layouts, stages, a render graph,
slots, input, a clock — and knows nothing about orbits, jumps or dialogue.

`shinra-engine/` is a separate git repository. It is read-only from here. If a
game cannot be expressed with the engine surface as it stands, that is a
missing primitive and a planning decision — not something to fix by teaching
the engine about your game.

## The layout rule

> **A `.rs` at depth 1 is a module. Anything deeper is a source file inside
> one.**

There is no `Cargo.toml`, no manifest and no registration table. The tree *is*
the configuration: the CLI walks it, and each depth-1 `.rs` becomes one `.so`.

Read [`game1/`](game1) before writing anything — it is the worked reference,
and it uses every part of the surface.

```
game1/
├── bundle/  data.rs  buffer.rs                    both required
├── process/ orbit.rs  spin.rs  breathe.rs
├── render/  solid.rs  ghost.rs  solid/shade.rs  wgsl/mod.rs
├── asset/   bunny.rs  teapot.rs  quad.rs  bunny/model.obj
└── game/    game1.rs  showcase.rs
```

A folder named after a sibling `.rs` is **private** to it:
[`render/solid/`](game1/render/solid) belongs to
[`render/solid.rs`](game1/render/solid.rs) alone, reached as `mod shade;`.
A folder with no matching sibling is **shared** across the category:
[`render/wgsl/`](game1/render/wgsl) is reachable from any render module as
`crate::render::wgsl`.

`bundle/` is the one exception to one-module-per-`.rs`:
[`data.rs`](game1/bundle/data.rs) and [`buffer.rs`](game1/bundle/buffer.rs)
compile together into a single `bundle.so`, because layouts and buffers are the
same contract and must version and reload as one. Both are required.

One entry point per category:

| file | entry point | what it declares |
|---|---|---|
| `bundle/data.rs` | `se::layouts!` | components — the whole contract |
| `bundle/buffer.rs` | `se::buffers!` | render targets, and which are sampled |
| `process/*.rs` | `se::stages!` | stages; the parameter list *is* the query |
| `render/*.rs` | `se::graph!` | passes and edges |
| `asset/*.rs` | `se::assets!` | name → bytes, nothing else |
| `game/*.rs` | `se::control!` | slots, `start`, `tick` |

Changing `bundle/` means a different game set and a full reload. Every other
module is hot-swappable on its own.

## Build and run

The CLI resolves the engine from `$SHINRA_ENGINE`, falling back to the
workspace the `shinra` binary was built from. Set it so a build never depends
on where the binary came from:

```bash
export SHINRA_ENGINE=/abs/path/to/shinra-engine     # set in the hive .env
```

Then, from this directory:

```bash
# compile every module of the bundle to a .so
"$SHINRA_ENGINE/target/debug/shinra" build game1

# build, then run it with the TUI IDE (needs a real terminal)
"$SHINRA_ENGINE/target/debug/shinra" run game1

# render headless and write a PPM to stdout
"$SHINRA_ENGINE/target/debug/shinra" shot game1 -n 3 --size 100x56 > /tmp/game1.ppm
```

`build` writes to `target/shinra/<bundle>`. Useful options: `--game <name>`
picks which `game/*.rs` drives (default: the first), `-o <dir>` moves the
output, `--release` optimises, `--no-watch` stops `run` rebuilding on change,
`-n <N>` limits frames, `--size <WxH>` sets the headless framebuffer.

A second control is selected the same way:

```bash
"$SHINRA_ENGINE/target/debug/shinra" shot game1 --game showcase -n 3 --size 100x56 > /tmp/showcase.ppm
```

`shot` is the check that works anywhere, including CI. `run` drives an
interactive TUI and will fail without a terminal.

### What counts as passing

Exit 0 and **zero** `Validation Error` lines on stderr. A frame that is one
flat colour means nothing drew — that is a failure even at exit 0. When a pass
misbehaves, `SE_DUMP_WGSL=/tmp/wgsl` writes each pass's generated shader.

Do not run `cargo` in the engine repository. The `shinra` binary is prebuilt
and builds `se` itself when it needs to.

## The games

### game1 — orbit

Bodies going round the middle, and the demonstration that swapping one module
changes exactly one thing.

- **The asset slot changes the model.** `bunny`, `teapot` and `quad` all
  publish the same name, `model.obj`. The render graph asks for one mesh; the
  control layer decides which module answers. Press **M**. Nothing in
  `render/` or `process/` learns it happened, and no orbit phase is lost.
- **The render slot changes the look.** `solid` draws one instanced mesh pass
  straight to the screen. `ghost` draws the same bodies over a decaying image
  of themselves, reading `trail` while writing `trail` — legal because a
  sampled buffer always resolves to the *previous* frame, so it is a one-frame
  feedback loop and not a cycle. Press **G**.
- **Two controls swap the rules.** `game/game1.rs` reads the keyboard;
  `game/showcase.rs` ignores it and cycles the themes on a timer. Same
  contract, same stages, same graph — different game.

Also in it: `orbit`, `spin` and `breathe` as three separate stages, so a body
can have one without the others; a `Camera` component named as a pass uniform,
which is how a camera reaches a shader without the render side being handed the
world; and a `Look` component holding the player's choices, because a `.so`
loses its `static`s when it is hot-swapped, so state that must survive is data.

Arrow keys orbit the view, **W**/**S** dolly, **+**/**-** change how many
bodies exist.

### game2 — dino (planned)

A 2D platformer in the shape of Chrome's no-network dinosaur: run, jump,
obstacles arriving from the right. Not in the tree yet.

### game3 — dialogue (planned)

An AVG-style dialogue game whose script is an *asset*, in the manner of Ren'Py
— so replacing one `asset/*.so` replaces the story while the game rules stay
put. Not in the tree yet.

## Things that will bite you

- A `.so` loses its `static`s when it is hot-swapped. State that must survive
  is a **component**.
- `start` runs again after a reload, so it must **reconcile** — compare what
  should exist against what does — never blindly spawn.
- A stage cannot spawn, cannot read assets, and cannot reach an entity outside
  its signature. That is deliberate.
- Only control writes input components and the roster. Content never writes
  data: an asset module adds a *definition*, a reconcile stage spawns.
- Shader-visible fields must be `f32`/`i32`/`u32` and must not be WGSL
  reserved words (`target`, `filter`, `sampler`, …). The engine names the
  offender.
- Uniform fields arrive as `vec4`: write `u.fov.x`, `u.eye.xyz`.
- A sampled buffer read is always the **previous** frame.

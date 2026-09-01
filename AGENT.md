# shinra-examples — agent instructions

Games for the Shinra Engine. **All game logic lives here.** The engine
provides general essential tools and nothing else.

## The one rule that matters

`shinra-engine/` is a **different git repository and is not yours to edit**.
Your worktree is this repo. If a ticket seems to need an engine change, that
is a planning decision, not something to work around:

> Stop. Set the ticket to `debugging`, and write in its doc exactly which
> engine primitive is missing and why.

Never add a component, stage, pass kind, key binding, physics constant,
sprite concept or dialogue concept to any `se-*` crate. The commit is
rejected automatically if the engine checkout is dirty.

## Layout — this is the whole configuration

**A `.rs` at depth 1 is a module. Anything deeper is a source file inside one.**

```
game1/
├── bundle/  data.rs  buffer.rs          both required
├── process/ orbit.rs  spin.rs  breathe.rs
├── render/  solid.rs  ghost.rs
│            wgsl/vs.wgsl  wgsl/light.wgsl        shared: no sibling .rs
│            solid/fs.wgsl                        private to solid.rs
│            ghost/bodies.fs.wgsl  ghost/trail.fs.wgsl
├── asset/   bunny.rs  teapot.rs  bunny/model.obj
└── game/    game1.rs  showcase.rs
```

## Shaders are `.wgsl` files, never strings in `.rs`

WGSL lives in `.wgsl` files and is pulled in with `include_str!`. A `.wgsl`
file is a source file inside a module, which is exactly what the layout rule
already says about anything deeper than depth 1.

```rust
se::graph!("solid", |g| g.present("scene").pass("bodies", |p| p
    .shader(concat!(
        include_str!("wgsl/vs.wgsl"),      // shared across render/
        include_str!("wgsl/light.wgsl"),
        include_str!("solid/fs.wgsl"),     // private to solid.rs
    ))
    .color(&["scene"]).depth("depth").uniform_of("Camera")
    .instanced("Sprite", "")));
```

Naming: `vs.wgsl` for a vertex stage, `fs.wgsl` for a fragment stage. When a
module has more than one pass, prefix with the pass name —
`bodies.fs.wgsl`, `trail.fs.wgsl`. Never put WGSL in a Rust string literal.

A folder named after a sibling `.rs` is private to it (`render/solid/`); one
without is shared across the category (`render/wgsl/`, reached as
`crate::render::wgsl`). There is no Cargo.toml and no manifest — the tree is
the configuration.

`game1/` is the worked reference. Read it before writing anything.

## Build and check

```bash
export SHINRA_ENGINE=/abs/path/to/shinra-engine     # set in the hive .env
"$SHINRA_ENGINE/target/debug/shinra" build <game>
"$SHINRA_ENGINE/target/debug/shinra" shot  <game> -n 3 --size 100x56 > /tmp/x.ppm
```

`build` compiles every module that exists, even when the bundle is not
complete yet — so **your module is checkable the moment you write it**, before
the rest of its story lands. It prints what the bundle still needs and exits 0.
That note is expected while a bundle is being written; it is not a failure and
it is not something for you to fix by inventing the missing modules.

`shot` needs a complete bundle (a `render/` and a `game/` module). If yours is
not complete yet, `build` is the whole of your verification — say so and stop.
Never go reverse-engineering the engine's `rustc` invocation to work around
this; if `build` cannot check your file, that is a tooling gap to report.

When the bundle is complete: exit 0, **zero** `Validation Error` lines on
stderr, and a frame that is not one flat colour. `SE_DUMP_WGSL=/tmp/wgsl`
writes each pass's generated shader when a pass misbehaves.

Do not run `cargo` in the engine repo. The `shinra` binary is already built.

## Things that will bite you

- A `.so` loses its `static`s when it is hot-swapped. State that must survive
  is a **component**.
- `start` runs again after a reload, so it must **reconcile**, never blindly
  spawn.
- A stage cannot spawn, cannot read assets, and cannot reach an entity outside
  its signature. That is deliberate.
- Shader-visible fields must be `f32/i32/u32` and must not be WGSL reserved
  words (`target`, `filter`, `sampler`, …). The engine names the offender.
- Uniform fields arrive as `vec4`: write `u.fov.x`, `u.eye.xyz`.
- A sampled buffer read is always the **previous** frame.

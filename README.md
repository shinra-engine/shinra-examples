# shinra

A small Rust + wgpu game engine that renders to either a native window or
into a terminal pane.

See [`design.md`](design.md) for the architecture rationale.

## Run

```bash
cargo run --bin window      # native winit window
cargo run --bin terminal    # half-block render inside any terminal (works in tmux)
```

In both binaries:

- **Arrow keys** — yaw / pitch the camera
- **j / k** — enlarge / shrink the bunny (terminal binary)
- **ESC / q** — quit

## Rendering pipeline (terminal binary)

```
bunny.obj → Mesh → GPU buffers → shader → offscreen RGBA texture → CPU buffer → RgbaImage → half-block in tmux
```

| Stage | Crate | What it does |
|---|---|---|
| Load `bunny.obj` from disk | `tobj` | Parses OBJ; gives vertex positions, indices, optional normals |
| Carry positions/normals as POD | `bytemuck` | Safe `&Vertex` ↔ `&[u8]` casts so wgpu can upload them |
| Math (camera, model, projection) | `glam` | `Vec3`, `Mat4`, `look_at_rh`, `perspective_rh`, `from_scale` |
| GPU device + render pipeline + Lambert shader (`shader.wgsl`) | `wgpu` | All actual rendering. Uses Vulkan backend on Linux. |
| Block-on async wgpu init | `pollster` | One-call `block_on` for `request_adapter` / `request_device` |
| Wrap GPU readback bytes as an image | `image` | `RgbaImage::from_raw(...)` so viuer accepts it |
| Print the image into the terminal cells | `viuer` | Auto-detects Kitty / iTerm2 / Sixel / half-block fallback; writes ANSI to stdout |
| Raw-mode + cursor positioning + arrow / j / k key polling | `crossterm` | `enable_raw_mode`, `MoveTo(0, 0)`, `event::poll` + `read` |
| `?` for OBJ load errors | `anyhow` | Boxed error type to keep `main` returns clean |

`winit` is in `Cargo.toml` but only the **window** binary uses it. The two
load-bearing crates for the terminal experience are **`wgpu`** (renders) and
**`viuer`** (turns rendered pixels into terminal output); everything else is
plumbing around those.

## Layout

```
src/
  lib.rs
  engine.rs           # wgpu device + offscreen render targets + render()
  scene.rs            # Scene, Camera, Drawable
  mesh.rs             # Vertex, Mesh, OBJ loader
  shader.wgsl         # geometry pipeline (Lambert)
  presenter/
    mod.rs            # Presenter trait + FrameCtx
    window.rs         # WindowPresenter (wgpu surface blit)
    blit.wgsl         # fullscreen-triangle blit
    terminal.rs       # TerminalPresenter (readback → viuer)
src/bin/
  window.rs           # winit ApplicationHandler
  terminal.rs         # crossterm raw-mode loop
assets/
  teapot.obj
  bunny.obj
tests/
  render_smoke.rs     # writes target/debug/smoke/{cube,teapot,bunny}.png
```

## Tests

```bash
cargo test                  # 9 unit + 1 smoke (writes PNGs)
ls target/debug/smoke/      # cube.png teapot.png bunny.png
```

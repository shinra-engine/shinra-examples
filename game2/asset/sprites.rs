//! `asset/sprites.wasm` — the sheet, as content.
//!
//! Nothing samples it yet: the flat pass draws tinted rectangles, because the
//! command set has no texture binding. The asset is published anyway, because
//! that is where it belongs and because the row that will name it already has
//! the `uv` field to do it with — the day a binding exists, this file does not
//! change.

#[used]
#[link_section = "se.asset"]
static DATA: [u8; include_bytes!("sprites/atlas.png").len()] =
    *include_bytes!("sprites/atlas.png");

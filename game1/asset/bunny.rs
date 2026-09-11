//! `asset/bunny.wasm` — content, and nothing else.
//!
//! No code: the bytes ride in a custom section and the host reads them out of
//! the file without instantiating anything. That is what lets an asset be
//! swapped with no module reloaded — the row that names it holds a `span`,
//! so a 2-triangle quad and a 5000-triangle teapot cost the same sixteen
//! bytes in the world.
//!
//! `#[used]` because nothing references it: `--gc-sections` would otherwise
//! drop the only thing in the module.

#[used]
#[link_section = "se.asset"]
static DATA: [u8; include_bytes!("bunny/model.obj").len()] = *include_bytes!("bunny/model.obj");

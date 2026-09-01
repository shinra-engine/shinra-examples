//! The render-target half of game1's contract.
//!
//! `scene` and `trail` are marked sampled, which is what allows a pass to read
//! them as textures. A sampled buffer always reads its *previous* frame, so
//! `render/ghost.rs` can read `trail` while writing `trail` — that is a
//! one-frame feedback loop, not a cycle.

use se::{BufferDesc, Format};

se::buffers!(
    // What the mesh pass draws into.
    BufferDesc::screen("scene", Format::Rgba8Unorm).sampled(),
    // Depth for that pass. Never sampled, so it stays single-buffered.
    BufferDesc::screen("depth", Format::Depth32Float),
    // Accumulator for the ghosting look. Same size as the screen.
    BufferDesc::screen("trail", Format::Rgba8Unorm).sampled(),
);

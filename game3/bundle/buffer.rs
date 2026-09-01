//! The render-target half of game3's contract.
//!
//! A visual novel is a flat stage: backdrop, portraits, dialogue box, glyphs.
//! Those are layers, and layers are pass order, so there is nothing here to
//! feed back and nothing to accumulate — `scene` is presented and that is the
//! whole graph's output.
//!
//! Deliberately *not* sampled: a sampled buffer always reads its previous
//! frame, which is the right tool for a mirror or a ghosting trail (see
//! `game1/bundle/buffer.rs`) and the wrong tool for text that must be legible
//! the frame it appears. The typewriter reveal is `Line.revealed` in
//! `data.rs` — data, not a one-frame blur. If a later look wants a genuine
//! cross-fade between lines, that is when a sampled target earns its place.

use se::{BufferDesc, Format};

se::buffers!(
    // Everything draws into this, and this is what the host shows.
    BufferDesc::screen("scene", Format::Rgba8Unorm),
    // Depth for the quad passes. A flat stage does not need it to sort, but
    // a pass that wants `LessEqual` has somewhere to write; never sampled,
    // so it stays single-buffered.
    BufferDesc::screen("depth", Format::Depth32Float),
);

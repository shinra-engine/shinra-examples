//! The render-target half of game2's contract.
//!
//! One target, because a flat game needs one. `render/flat.rs` presents
//! `scene`; the runner, the cacti and the ground are drawn into it back to
//! front, so there is no depth buffer — sorting is the draw order, and a
//! depth attachment nothing tests against is dead weight in the contract.
//!
//! Nothing is `sampled` either. A sampled buffer is double-buffered so a pass
//! can read last frame's copy of it; game2 has no feedback pass, so paying
//! for the second image would buy nothing.

use se::{BufferDesc, Format};

se::buffers!(
    // What the flat pass draws into, and what is presented.
    BufferDesc::screen("scene", Format::Rgba8Unorm),
);

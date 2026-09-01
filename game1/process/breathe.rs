//! A slow scale wobble.
//!
//! The smallest stage in the bundle, and the point of it is that it is small:
//! adding a behaviour means adding a file, not editing a scheduler, a system
//! list, or a registration table. The host discovers it because it is a `.rs`
//! at depth 1 of `process/`.

use data::{Breathe, Transform};

#[se::stage]
fn breathe(t: &mut Transform, b: &mut Breathe, dt: f32) {
    b.phase += b.rate * dt;
    let k = b.base + b.amount * b.phase.sin();
    t.scale = [k, k, k];
}

se::stages!(breathe);

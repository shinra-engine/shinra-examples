//! A slow scale wobble, so a still frame still reads as alive.
//!
//! The smallest possible stage: two columns, one field written. It is here
//! mostly to show that adding a behaviour is adding a file — the host is not
//! recompiled, the contract does not move, and nothing lists it.

use crate::process::math::sin;
use crate::*;

#[se::stage]
fn breathe(b: &mut Breathe, t: &mut Transform) {
    let dt = unsafe { (*(arena::CLOCK as *const Clock)).dt };
    b.phase += b.rate * dt;
    let k = b.base + sin(b.phase) * b.amount;
    t.scale = [k, k, k];
}

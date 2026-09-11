//! What makes a thing go round the middle.
//!
//! `phase` is integrated here and the result is written into `Transform.pos`;
//! nothing else may write it. That is not a convention — the signature is the
//! whole query, so reaching outside it is not expressible.

use crate::process::math::{cos, sin};
use crate::*;

/// Time is a control var now, not a parameter. A stage takes no arguments at
/// all: everything it reads is at an address the contract fixed.
fn clock() -> Clock {
    unsafe { *(arena::CLOCK as *const Clock) }
}

#[se::stage]
fn orbit(o: &mut Orbit, t: &mut Transform) {
    let dt = clock().dt;
    o.phase += o.speed * dt;

    let (s, c) = (sin(o.phase), cos(o.phase));
    let tilt = o.tilt;
    t.pos = [
        o.center[0] + c * o.radius,
        o.center[1] + s * o.radius * sin(tilt),
        o.center[2] + s * o.radius * cos(tilt),
    ];
}

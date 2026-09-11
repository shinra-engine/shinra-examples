//! Rotation about the body's own axis, independent of where it is.
//!
//! It writes `Transform.rot` and never `Transform.pos`, while `orbit.rs`
//! writes `pos` and never `rot`. Both name `&mut Transform`, and that is
//! allowed: a pool's rows are the same rows, and single-writer is a property
//! of fields here rather than of whole records.

use crate::process::math::{cos, sin, sqrt};
use crate::*;

fn clock() -> Clock {
    unsafe { *(arena::CLOCK as *const Clock) }
}

#[se::stage]
fn spin(s: &mut Spin, t: &mut Transform) {
    s.angle += s.rate * clock().dt;

    // A quaternion from axis and angle. The axis arrives normalised; a body
    // whose axis is zero simply does not turn.
    let half = s.angle * 0.5;
    let (sn, cs) = (sin(half), cos(half));
    let len = sqrt(s.axis[0] * s.axis[0] + s.axis[1] * s.axis[1] + s.axis[2] * s.axis[2]);
    if len > 1e-6 {
        let k = sn / len;
        t.rot = [s.axis[0] * k, s.axis[1] * k, s.axis[2] * k, cs];
    }
}


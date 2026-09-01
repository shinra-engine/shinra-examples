//! Going round the middle.
//!
//! The signature is the query: every entity that has both a `Transform` and an
//! `Orbit` is visited, and nothing else is reachable from in here. There is no
//! world handle to misuse and no way to touch an entity this function was not
//! handed.

use data::{Orbit, Transform};

/// Advance the orbital phase and place the body on its ring.
///
/// `Transform.pos` is written here and by nothing else in the bundle, which is
/// what makes "who moved this?" answerable by reading one file.
#[se::stage]
fn orbit(t: &mut Transform, o: &mut Orbit, dt: f32) {
    o.phase += o.speed * dt;
    if o.phase > std::f32::consts::TAU {
        o.phase -= std::f32::consts::TAU;
    }

    let (s, c) = o.phase.sin_cos();
    let (ts, tc) = o.tilt.sin_cos();

    // Ring in XZ, then tipped about the X axis by `tilt`.
    let x = c * o.radius;
    let y = 0.0;
    let z = s * o.radius;

    t.pos = [
        o.center[0] + x,
        o.center[1] + y * tc - z * ts,
        o.center[2] + y * ts + z * tc,
    ];
}

se::stages!(orbit);

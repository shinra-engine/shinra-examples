//! Rotation about a body's own axis.
//!
//! Separate from `orbit` on purpose: where a thing *is* and which way it
//! *faces* are different questions, and keeping them in different stages means
//! a body can have one without the other. A quad that orbits without spinning
//! just has no `Spin`.

use data::{Spin, Transform};

#[se::stage]
fn spin(t: &mut Transform, s: &mut Spin, dt: f32) {
    s.angle += s.rate * dt;
    if s.angle > std::f32::consts::TAU {
        s.angle -= std::f32::consts::TAU;
    }

    let n = norm(s.axis);
    let half = s.angle * 0.5;
    let sin = half.sin();
    // Unit quaternion, xyzw — the layout `Transform` promised.
    t.rot = [n[0] * sin, n[1] * sin, n[2] * sin, half.cos()];
}

fn norm(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 1e-6 {
        [v[0] / l, v[1] / l, v[2] / l]
    } else {
        [0.0, 1.0, 0.0]
    }
}

se::stages!(spin);

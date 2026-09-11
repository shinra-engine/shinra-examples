//! What falls, and what stands on the ground.
//!
//! Two columns of the same pool, so row `i` is one thing in both. The runner
//! is the only row with a velocity that matters; the rest carry a `body` they
//! never use, which is what a pool costs — twelve bytes a row, against an
//! indirection on every access if they did not share a length.

use crate::*;

/// The floor, in world units. A flat game has one and it is a constant, not a
/// collision mesh.
const GROUND: f32 = -1.4;
const G: f32 = -26.0;

#[se::stage]
fn gravity(b: &mut Body, s: &mut Sprite) {
    // A row with no mass is not a faller. `on_ground` is 2 for scenery, which
    // is how a pool says "this column does not apply to me" without a second
    // pool or a branch in the host.
    if b.on_ground == 2 {
        return;
    }
    let dt = unsafe { (*(arena::CLOCK as *const Clock)).dt };

    b.vel[1] += G * dt;
    s.pos[1] += b.vel[1] * dt;

    let floor = GROUND + s.size[1] * 0.5;
    if s.pos[1] <= floor {
        s.pos[1] = floor;
        b.vel[1] = 0.0;
        b.on_ground = 1;
    } else {
        b.on_ground = 0;
    }
}

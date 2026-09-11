//! The world moving past the runner.
//!
//! A hazard is never despawned. It scrolls left, and once it is off the left
//! edge it parks itself with `live == 0` and waits — which is how a fixed
//! roster behaves like an endless one. Nothing spawns, so nothing has to.

use crate::*;

/// Where a hazard is considered gone, and where control puts it back.
const LEFT_EDGE: f32 = -6.0;

#[se::stage]
fn scroll(h: &mut Hazard, s: &mut Sprite) {
    if h.live != 1 {
        return;
    }
    let dt = unsafe { (*(arena::CLOCK as *const Clock)).dt };
    s.pos[0] -= h.speed * dt;
    if s.pos[0] < LEFT_EDGE {
        h.live = 0;
        // Parked off-screen, so a frame drawn before control hands it back
        // out shows nothing rather than a cactus at the origin.
        s.pos[0] = LEFT_EDGE - 4.0;
    }
}

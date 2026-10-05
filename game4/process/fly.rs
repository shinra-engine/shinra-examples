//! Everything that moves, moves here.
//!
//! One stage over the whole pool: a row with a velocity is carried along it,
//! and what happens at the edge of the screen depends on what the row is. A
//! bullet that leaves the top and a bomb that leaves the bottom park
//! themselves, so control can hand them out again; a star that leaves the
//! bottom comes back in at the top. The bees are moved here too — control only
//! sets their heading — and the ship is not, because it has no velocity.

use crate::*;

const KIND_BULLET: u32 = 2;
const KIND_BOMB: u32 = 4;
const KIND_STAR: u32 = 5;

/// The top and bottom of the playfield, with a margin so a sprite is fully off
/// the screen before it is parked.
const TOP: f32 = 2.5;
const BOTTOM: f32 = -2.5;

#[se::stage]
fn fly(a: &mut Actor, s: &mut Sprite) {
    if a.live != 1 {
        return;
    }
    // Once the run is won or lost the board holds still; only the stars drift.
    let alive = unsafe { (*(arena::RUN as *const Run)).alive };
    if alive != 1 && a.kind != KIND_STAR {
        return;
    }
    let dt = unsafe { (*(arena::CLOCK as *const Clock)).dt };
    s.pos[0] += a.vel[0] * dt;
    s.pos[1] += a.vel[1] * dt;

    let gone = match a.kind {
        KIND_BULLET => s.pos[1] > TOP,
        KIND_BOMB => s.pos[1] < BOTTOM,
        KIND_STAR => {
            if s.pos[1] < BOTTOM + 0.1 {
                s.pos[1] = TOP - 0.1;
            }
            false
        }
        _ => false,
    };
    if gone {
        // Parked, and drawn at zero size, so a frame drawn before control
        // hands the row back out shows nothing.
        a.live = 0;
        s.size = [0.0, 0.0];
        s.pos[1] = -9.0;
    }
}

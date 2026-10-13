//! Every collision is decided here, over every pair of rows.
//!
//! Stages run in name order, so `fly` moves everything and then `hit` tests
//! the new positions: motion precedes collision within a frame. Control reads
//! the columns on the next tick and derives `bees`, `score` and `alive`.

use crate::*;

// Duplicated from game/game4.rs: modules share nothing across categories.
const K_SHIP: u32 = 1;
const K_BULLET: u32 = 2;
const K_BEE: u32 = 3;
const K_BOMB: u32 = 4;

#[se::pair]
fn hit((a, sa): (&mut Actor, &mut Sprite), (b, sb): (&mut Actor, &mut Sprite)) {
    if a.live != 1 || b.live != 1 {
        return;
    }
    // One pool visits i<j once, so accept both orders.
    let (x, sx, y, sy) = match (a.kind, b.kind) {
        (K_BULLET, K_BEE) | (K_BOMB, K_SHIP) | (K_BEE, K_SHIP) => (a, sa, b, sb),
        (K_BEE, K_BULLET) | (K_SHIP, K_BOMB) | (K_SHIP, K_BEE) => (b, sb, a, sa),
        _ => return,
    };
    if !overlap(sx, sy) {
        return;
    }
    if x.kind == K_BULLET {
        // The bullet and the bee both go.
        park(x, sx);
        park(y, sy);
    } else {
        // The ship is hit.
        y.live = 0;
    }
}

/// Centre-distance test against half-extents, which is why `pos` is the centre.
fn overlap(a: &Sprite, b: &Sprite) -> bool {
    (a.pos[0] - b.pos[0]).abs() < (a.size[0] + b.size[0]) * 0.5
        && (a.pos[1] - b.pos[1]).abs() < (a.size[1] + b.size[1]) * 0.5
}

/// Take a row out of play: not moved, not tested, and nothing to draw.
fn park(a: &mut Actor, s: &mut Sprite) {
    a.live = 0;
    s.size = [0.0, 0.0];
    s.pos = [0.0, -9.0];
}

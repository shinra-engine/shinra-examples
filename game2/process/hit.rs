//! The runner meeting a cactus, decided over every pair of rows.
//!
//! A hit is a mark, not a verdict: the cactus's `live` becomes 2 and control
//! reads it on the next tick to end the run. Scenery has `on_ground == 2`, a
//! cactus has `live == 1`, the runner is the row that is neither.

use crate::*;

const LIVE: u32 = 1;
const HIT: u32 = 2;
const SCENERY: u32 = 2;

#[se::pair]
fn hit(
    (ba, sa, ha): (&Body, &Sprite, &mut Hazard),
    (bb, sb, hb): (&Body, &Sprite, &mut Hazard),
) {
    // One pool visits i<j once, so either side may be the cactus.
    let cactus = if ha.live == LIVE && ba.on_ground == SCENERY && hb.live == 0 && bb.on_ground != SCENERY {
        ha
    } else if hb.live == LIVE && bb.on_ground == SCENERY && ha.live == 0 && ba.on_ground != SCENERY {
        hb
    } else {
        return;
    };
    // Centre-distance test against half-extents, which is why `pos` is the centre.
    if (sa.pos[0] - sb.pos[0]).abs() < (sa.size[0] + sb.size[0]) * 0.5
        && (sa.pos[1] - sb.pos[1]).abs() < (sa.size[1] + sb.size[1]) * 0.5
    {
        cactus.live = HIT;
    }
}

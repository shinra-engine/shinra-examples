//! The actor rows: one per cast member, laid out left to right.
//!
//! Private to `game/game3.rs`. Rows are cheap, so they are rewritten every
//! tick; that keeps the roster right after an act swap.

use crate::*;

/// Set the pool length to `cast` and write each actor row.
pub unsafe fn write(cast: u32) {
    (*se_pool_mut(arena::POOL_SPRITES)).len = cast;
    let cs = se_col(arena::COL_SPRITE);
    for k in 0..cast {
        let s = &mut *se_row::<Sprite>(&cs, k);
        let t = (k as f32 + 0.5) / cast as f32;
        s.kind = 0;
        s.pos = [(t * 2.0 - 1.0) * 1.6 * 0.62, 0.05];
        s.size = [0.34, 0.52];
        s.uv = [k as f32, 0.0, 1.0, 1.0];
    }
}

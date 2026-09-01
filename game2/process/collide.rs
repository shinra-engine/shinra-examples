//! Hit detection: has the runner run into this cactus?
//!
//! The engine will only ever hand a stage one entity's row at a time, and the
//! parameter list is the query — so "compare the runner with every hazard" is
//! not a shape a single stage can have. There is no world handle, no way to
//! name a second entity, and `resolve` in the host intersects the columns a
//! signature names rather than joining them, so a signature that asked for
//! both the runner's `Body` and a cactus's `Hazard` would match nothing at all.
//!
//! ## The shape chosen, and why
//!
//! The test is split along the axis that decides which station can express it:
//!
//! - **x is a game constant.** The runner never travels; the desert scrolls
//!   past it (`process/scroll.rs`). Its column is therefore a *number*, not
//!   another entity's field, and the horizontal half of the overlap test is
//!   fully expressible from inside a stage over hazards. That is this file.
//! - **y is another entity's field.** The runner's height is mid-jump state
//!   living in its own `Sprite.pos[1]`, which a hazard row cannot see. The one
//!   remaining comparison — the runner's box against `[pos[1] - height * 0.5,
//!   pos[1] + height * 0.5]` of a flagged cactus — belongs to `game/game2.rs`,
//!   the only station that can hold two entities at once, and it is also the
//!   station that owns `Run.alive`.
//!
//! The alternative shapes were considered and rejected. Caching the runner's y
//! in a `static` does not fail on the hot-swap rule alone — it fails earlier:
//! this stage's rows are the entities carrying *both* `Hazard` and `Sprite`,
//! so the runner, which has no `Hazard`, is never visited here and there is no
//! frame in which such a `static` could be filled. Smuggling the value through
//! `Body._pad` would give a documented padding field a second meaning that
//! `bundle/data.rs` does not claim.
//!
//! ## Ordering, for whoever writes `game/game2.rs`
//!
//! The host runs control's `tick` *before* the frame's stages, so control
//! always reads the flag this stage wrote during the **previous** frame. That
//! is a one-frame latency, not a missed hit: a cactus stays in the runner's
//! column for several frames at the speeds `Run.speed` ramps through, so the
//! flag is still set when control next looks. It does mean the x half of the
//! test is one frame stale while the y half control does is fresh — worth
//! knowing only if the ramp is ever pushed fast enough that a cactus crosses
//! the whole column, `2 * reach` wide, inside a single frame.
//!
//! ## The flag
//!
//! `Hazard.active` carries it. A cactus is parked (0) or in play (1), and this
//! stage adds a third reading — 2, *in play and standing in the runner's
//! column* — so no new field and no new component is needed for the outcome.
//! The value is recomputed from scratch every frame, so the flag clears itself
//! the moment the cactus scrolls clear and cannot survive a restart as a stale
//! game-over.

use data::{Hazard, Sprite};

/// Parked at the right edge, waiting to be released. Not in play, so not a
/// threat. `process/scroll.rs` writes this when a cactus leaves the screen.
const PARKED: u32 = 0;

/// In play, and clear of the runner's column.
const IN_PLAY: u32 = 1;

/// In play, and horizontally overlapping the runner. `game/game2.rs` reads
/// this, checks the one axis a stage cannot, and writes `Run.alive = 0`.
const ALIGNED: u32 = 2;

/// World x of the runner's centre.
///
/// The second number in this bundle that has to agree with something outside
/// the file it lives in — `game/game2.rs` spawns the runner here, exactly as
/// `WORLD_HALF_W` in `process/scroll.rs` has to agree with the `View` the flat
/// pass is given. It is a constant rather than a read of the runner's
/// `Sprite`, because taking `&Sprite` for the runner would make this stage a
/// query against the runner and it would then never see a cactus.
///
/// Left of centre, the way the no-network runner puts it: the cacti come from
/// the right and there has to be enough screen between them and the runner to
/// react.
const PLAYER_X: f32 = -12.0;

/// Half the runner's *hitbox* width, world units.
///
/// Deliberately narrower than the runner is drawn, for the same reason
/// `Hazard.width` is its own field and not `Sprite.size[0]`: a clipped corner
/// that ends a run reads as unfair, so both boxes are forgiving and the
/// generosity is a number a reader can find.
const PLAYER_HALF_W: f32 = 0.7;

/// Flag every in-play hazard that is standing in the runner's column.
///
/// `Hazard.active` is written here in the 1 ↔ 2 directions only. The 1 → 0
/// retirement stays in `process/scroll.rs` and the 0 → 1 release stays in
/// `game/game2.rs`, so the field has exactly one writer per transition and
/// "who parked this cactus?" is still answerable by reading one file.
///
/// `Sprite` is taken by shared reference: hit detection reads geometry and
/// changes none of it. A crash moves nothing — what it does is end the run,
/// and that is a `Run` field in the station that owns it.
#[se::stage]
fn collide(h: &mut Hazard, s: &Sprite) {
    // A parked cactus sits off the right edge and must stay parked; flagging
    // it would hand control a hit for an obstacle that is not in the game yet.
    if h.active == PARKED {
        return;
    }

    // Axis-aligned overlap on x, centre-to-centre: `Sprite.pos` is a centre by
    // the definition in `bundle/data.rs`, so the boxes touch when the distance
    // between the centres is under the sum of the half-widths. No corner
    // arithmetic, and no origin convention to get wrong.
    let gap = (s.pos[0] - PLAYER_X).abs();
    let reach = h.width * 0.5 + PLAYER_HALF_W;

    // Assigned, not or-ed: recomputing the whole flag every frame is what
    // makes a cactus that has scrolled past the runner stop being a threat
    // without anybody having to clear it.
    h.active = if gap < reach { ALIGNED } else { IN_PLAY };
}

se::stages!(collide);

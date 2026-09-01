//! The world moving past.
//!
//! In the no-network runner the runner never travels: it jumps in place and
//! the desert slides underneath it. So this is the stage that makes the game
//! a game — `process/gravity.rs` owns the y axis, this file owns the x axis,
//! and neither can reach the other's field because neither was handed the
//! other's component.
//!
//! The signature is the query: every entity with both a `Sprite` and a
//! `Hazard` is visited, and nothing else exists from in here. The runner has
//! no `Hazard`, so it cannot be dragged off the left of the screen by a bug
//! in this file — not because a branch skips it, but because it is not
//! reachable.

use data::{Hazard, Sprite};

/// Half-width of the visible world, in the same units `Sprite.pos` is in.
///
/// This is the one number in the file that has to agree with something
/// outside it — the `View` the flat pass is given. It is written here rather
/// than read from `View` because a stage may only touch the components in its
/// signature, and taking `&View` would make every cactus a query against the
/// camera. Widen the camera and this widens with it; the cost of the
/// duplication is that the two must be changed together, and the benefit is
/// that a stage stays a pure function of the row it was handed.
const WORLD_HALF_W: f32 = 20.0;

/// Where a recycled cactus is parked, as a distance beyond the right edge.
///
/// Far enough out that a hazard control reactivates is never seen to pop into
/// existence mid-screen, and the gap is measured from the *edge* rather than
/// from a fixed x so it survives a change to `WORLD_HALF_W`.
const STAGING_GAP: f32 = 6.0;

/// Slide every live hazard leftwards, and park the ones that have left.
///
/// `Sprite.pos[0]` of a hazard is written here and by nothing else in the
/// bundle. `Hazard.active` is written here *only* in the 1 → 0 direction: a
/// stage cannot spawn, so retiring an obstacle that has gone past is this
/// file's job, while handing one back out is `game/game2.rs`'s. That split is
/// what keeps the entity count flat — the roster is fixed at start and a
/// cactus is a slot that is either in play or waiting at the right edge, never
/// a thing that is created and destroyed.
#[se::stage]
fn scroll(s: &mut Sprite, h: &mut Hazard, dt: f32) {
    // A parked cactus is already sitting at the staging point and must stay
    // there: moving it while it waits would mean control could only release
    // one within a narrow window, and after that the slot would be useless.
    if h.active == 0 {
        return;
    }

    // `speed` is leftward by the definition in `bundle/data.rs`, so it is
    // subtracted. Per-hazard rather than global because the ramp in `Run`
    // reaches a cactus by being written into it when it is released — a stage
    // that cannot see `Run` still scrolls at the current pace.
    s.pos[0] -= h.speed * dt;

    // "Left the screen" is the *trailing* edge clearing the left edge, using
    // the drawn width so a wide cactus is not clipped away while half of it is
    // still visible. Recycled to the right rather than left in place: the next
    // release then costs control one field, and nothing on screen jumps.
    if s.pos[0] + s.size[0] * 0.5 < -WORLD_HALF_W {
        s.pos[0] = WORLD_HALF_W + STAGING_GAP + s.size[0] * 0.5;
        h.active = 0;
    }
}

se::stages!(scroll);

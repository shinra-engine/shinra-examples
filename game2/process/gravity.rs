//! Falling, and the one jump you are allowed.
//!
//! The signature is the query: an entity is visited only if it has both a
//! `Sprite` and a `Body`, and in game2 only the runner has a `Body`. The
//! cacti fall under no gravity because they were never handed to this
//! function, not because a branch in here skips them — there is no way from
//! inside a stage to reach an entity the parameter list did not name.
//!
//! Every number the jump feels like is in this file. That is the whole point:
//! the engine has no physics constant, `asset/sprites.rs` has no rule, and
//! "why does it float?" is answered by reading the four constants below.

use data::{Body, Sprite};

/// World y of the ground surface. A sprite's `pos` is its centre, so a body
/// rests with its *bottom* here, at `GROUND_Y + size[1] * 0.5` — the clamp is
/// written in terms of the sprite's own height and so does not care how tall
/// the runner is drawn.
const GROUND_Y: f32 = 0.0;

/// Downward acceleration, world units per second squared.
const GRAVITY: f32 = -60.0;

/// Upward speed the moment a jump starts, world units per second. With
/// `GRAVITY` that is an apex of `JUMP_SPEED^2 / (2 * -GRAVITY)` ≈ 4 units and
/// about 0.73 s in the air — long enough to clear a cactus, short enough that
/// the run still feels twitchy.
const JUMP_SPEED: f32 = 22.0;

/// Fall speed is capped so a long drop still lands on the frame it should,
/// rather than stepping past the ground line and being yanked back.
const MAX_FALL: f32 = -80.0;

/// Integrate the runner's vertical state and own `Body.on_ground`.
///
/// `Body.vel[1]`, `Sprite.pos[1]` and `on_ground` are written here and by
/// nothing else in the bundle, so a jump that misbehaves has exactly one file
/// to blame. `vel[0]` is left alone — the world scrolls past the runner in
/// `process/scroll.rs`; the runner does not travel.
#[se::stage]
fn gravity(s: &mut Sprite, b: &mut Body, i: &se::Input, dt: f32) {
    // Take-off is an edge, not a hold: leaning on Space does not hop, and the
    // `on_ground` test is the only thing keeping a second jump out of the air.
    let jumped = i.just_pressed(se::key::SPACE) || i.just_pressed(se::key::UP);
    if b.on_ground != 0 && jumped {
        b.vel[1] = JUMP_SPEED;
        b.on_ground = 0;
    }

    // Semi-implicit Euler: the new velocity moves the sprite, which keeps the
    // apex stable as `dt` wobbles between frames.
    b.vel[1] += GRAVITY * dt;
    if b.vel[1] < MAX_FALL {
        b.vel[1] = MAX_FALL;
    }
    s.pos[1] += b.vel[1] * dt;

    // The floor. Landing kills the downward velocity so the next frame does
    // not accumulate a fall the runner never took.
    let rest = GROUND_Y + s.size[1] * 0.5;
    if s.pos[1] <= rest {
        s.pos[1] = rest;
        b.vel[1] = 0.0;
        b.on_ground = 1;
    }
}

se::stages!(gravity);

//! Control for game2: the no-network runner.
//!
//! The only station that knows the time. Its write surface is what the design
//! allows — the input components (`View`, `Run`) and the roster — and it is
//! careful not to reach past that: `Sprite.pos[1]` for the runner belongs to
//! `process/gravity.rs`, and `Sprite.pos[0]` for a cactus belongs to
//! `process/scroll.rs`. Control touches those only when it is *creating* or
//! *resetting* an entity, which is roster work, not simulation.
//!
//! # The numbers that live in two files
//!
//! Three constants here have to agree with the stages, and the stages say so
//! in their own comments:
//!
//! | here | there | why |
//! |---|---|---|
//! | `PLAYER_X` | `process/collide.rs` | the column collision is measured against |
//! | `WORLD_HALF_W` | `process/scroll.rs` | where a cactus retires and re-stages |
//! | `GROUND_Y` | `process/gravity.rs`, `render/flat/sky.fs.wgsl` | what "the ground" means |
//!
//! A stage cannot ask control for a number and control cannot call a stage, so
//! a shared constant is the honest way to say it. The alternative — putting
//! them in `bundle/data.rs` as a component — would make them data the player
//! could be given, which they are not.
//!
//! # Who ends a run
//!
//! `process/collide.rs` flags a cactus `ALIGNED` when it shares the runner's
//! column. That is one axis. It cannot check the other, because a stage sees
//! one entity at a time and the runner is a different entity — so the vertical
//! test happens here, where both are reachable. Alignment is not a hit.

use data::{Body, Hazard, Run, Sprite, View};
use se::{key, Ctl, Entity, Frame};

// --- agreed with the stages ------------------------------------------------
/// World x of the runner. `process/collide.rs` measures against this.
const PLAYER_X: f32 = -12.0;
/// Half-width of the visible world. `process/scroll.rs` retires past this.
const WORLD_HALF_W: f32 = 20.0;
/// World y of the ground surface.
const GROUND_Y: f32 = 0.0;
/// Matches `STAGING_GAP` in `process/scroll.rs`: where a parked cactus waits.
const STAGING_GAP: f32 = 6.0;

// --- this game's own numbers -----------------------------------------------
const RUNNER_SIZE: [f32; 2] = [1.3, 1.9];
const RUNNER_TINT: [f32; 4] = [0.20, 0.22, 0.26, 1.0];
const CACTUS_TINT: [f32; 4] = [0.16, 0.42, 0.24, 1.0];

/// A fixed pool. A stage cannot spawn, so cacti are never created in flight —
/// they are parked off-screen and handed back out. The roster is constant from
/// the first frame, which is also why a hot reload cannot double it.
const POOL: u32 = 6;

/// Cactus sizes, cycled through the pool so the field is not uniform.
const CACTI: [[f32; 2]; 3] = [[0.8, 1.5], [1.1, 2.2], [0.7, 1.1]];

const START_SPEED: f32 = 11.0;
const MAX_SPEED: f32 = 26.0;
/// Metres per second, per second. Gentle: the run should outlast the novelty.
const RAMP: f32 = 0.35;
/// Seconds between releases at `START_SPEED`, scaled down as it ramps.
const BASE_GAP: f32 = 1.5;

// ---------------------------------------------------------------------------

/// Runs once at load, and again after every hot reload — so it reconciles what
/// should exist against what does, rather than spawning. Running it twice is a
/// no-op, which is what makes swapping this module mid-run safe.
fn start(c: &mut Ctl) {
    ensure_run(c);
    ensure_view(c);
    ensure_runner(c);
    ensure_cacti(c);
    c.log("game2: roster reconciled");
}

fn tick(c: &mut Ctl, f: &Frame) {
    let input = unsafe { f.input() };

    // The view is an input component: control may write it, nothing else may.
    // Half-height follows the framebuffer so the world does not stretch when
    // the terminal is resized.
    if let Some((e, mut v)) = first::<View>(c, "View") {
        v.half_w = WORLD_HALF_W;
        v.half_h = WORLD_HALF_W * f.height.max(1) as f32 / f.width.max(1) as f32;
        v.cx = 0.0;
        v.cy = v.half_h - 2.0;
        c.set(e, "View", &v);
    }

    let Some((run_e, mut run)) = first::<Run>(c, "Run") else { return };

    if run.alive != 0 {
        run.speed = (run.speed + RAMP * f.dt).min(MAX_SPEED);
        // Score is distance, so it reads as "how far", not "how long".
        run.score = run.score.saturating_add((run.speed * f.dt) as u32);
        run.spawn_t -= f.dt;
        if run.spawn_t <= 0.0 {
            release(c, run.speed);
            // Faster world, tighter gaps — but never closer than a jump can clear.
            run.spawn_t = (BASE_GAP * START_SPEED / run.speed).max(0.75);
        }
        if struck(c) {
            run.alive = 0;
            c.log("game2: crashed");
        }
    } else if input.just_pressed(key::SPACE) || input.just_pressed(key::ENTER) {
        reset(c);
        run = Run { alive: 1, score: 0, speed: START_SPEED, spawn_t: BASE_GAP };
    }

    c.set(run_e, "Run", &run);

    if input.swipe != 0 {
        c.swipe(input.swipe as i32);
    }
}

/// The vertical half of collision — the half a stage cannot do.
///
/// `process/collide.rs` has already narrowed the field to cacti sharing the
/// runner's column; all that is left is whether the runner is above the one it
/// is sharing it with.
fn struck(c: &mut Ctl) -> bool {
    const ALIGNED: u32 = 2;
    let Some((_, runner)) = first::<Sprite>(c, "Body") else { return false };
    let foot = runner.pos[1] - runner.size[1] * 0.5;

    let mut ids = [0u64; 32];
    let n = c.query("Hazard", &mut ids).min(ids.len() as u32);
    for &e in ids.iter().take(n as usize) {
        let (Some(h), Some(s)) = (c.get::<Hazard>(e, "Hazard"), c.get::<Sprite>(e, "Sprite"))
        else {
            continue;
        };
        if h.active == ALIGNED && foot < s.pos[1] + h.height * 0.5 {
            return true;
        }
    }
    false
}

/// Hand out a parked cactus. `process/scroll.rs` already left it at the
/// staging position when it retired, so releasing it is one field.
fn release(c: &mut Ctl, speed: f32) {
    let mut ids = [0u64; 32];
    let n = c.query("Hazard", &mut ids).min(ids.len() as u32);
    for &e in ids.iter().take(n as usize) {
        let Some(mut h) = c.get::<Hazard>(e, "Hazard") else { continue };
        if h.active == 0 {
            h.active = 1;
            h.speed = speed;
            c.set(e, "Hazard", &h);
            return;
        }
    }
    // Pool exhausted: the field is already full, so skipping a release is the
    // right answer. Growing the roster mid-run would make the reload path lie.
}

/// Put the world back to how `start` left it, without disturbing the roster.
fn reset(c: &mut Ctl) {
    if let Some((e, mut s)) = first::<Sprite>(c, "Body") {
        s.pos = [PLAYER_X, GROUND_Y + RUNNER_SIZE[1] * 0.5];
        c.set(e, "Sprite", &s);
        c.set(e, "Body", &Body { vel: [0.0; 2], on_ground: 1, _pad: 0 });
    }
    let mut ids = [0u64; 32];
    let n = c.query("Hazard", &mut ids).min(ids.len() as u32);
    for (i, &e) in ids.iter().take(n as usize).enumerate() {
        park(c, e, i as u32);
    }
}

fn park(c: &mut Ctl, e: Entity, i: u32) {
    let size = CACTI[i as usize % CACTI.len()];
    let mut h = c.get::<Hazard>(e, "Hazard").unwrap_or(Hazard {
        speed: START_SPEED,
        width: size[0],
        height: size[1],
        active: 0,
    });
    h.active = 0;
    h.width = size[0];
    h.height = size[1];
    c.set(e, "Hazard", &h);
    c.set(
        e,
        "Sprite",
        &Sprite {
            // Where `process/scroll.rs` re-stages a retired cactus, so a parked
            // one and a retired one are in the same place by construction.
            pos: [
                WORLD_HALF_W + STAGING_GAP + size[0] * 0.5 + i as f32 * 3.0,
                GROUND_Y + size[1] * 0.5,
            ],
            size,
            tint: CACTUS_TINT,
        },
    );
}

// --- reconcile helpers -----------------------------------------------------

fn ensure_run(c: &mut Ctl) {
    if first::<Run>(c, "Run").is_none() {
        let e = c.spawn();
        c.set(
            e,
            "Run",
            &Run { alive: 1, score: 0, speed: START_SPEED, spawn_t: BASE_GAP },
        );
    }
}

fn ensure_view(c: &mut Ctl) {
    if first::<View>(c, "View").is_none() {
        let e = c.spawn();
        c.set(
            e,
            "View",
            &View { half_w: WORLD_HALF_W, half_h: 11.0, cx: 0.0, cy: 9.0 },
        );
    }
}

fn ensure_runner(c: &mut Ctl) {
    if first::<Sprite>(c, "Body").is_some() {
        return;
    }
    let e = c.spawn();
    c.set(
        e,
        "Sprite",
        &Sprite {
            pos: [PLAYER_X, GROUND_Y + RUNNER_SIZE[1] * 0.5],
            size: RUNNER_SIZE,
            tint: RUNNER_TINT,
        },
    );
    c.set(e, "Body", &Body { vel: [0.0; 2], on_ground: 1, _pad: 0 });
}

fn ensure_cacti(c: &mut Ctl) {
    let mut ids = [0u64; 32];
    let have = c.query("Hazard", &mut ids);
    for i in have..POOL {
        let e = c.spawn();
        // `park` writes both components, so a fresh entity and a retired one
        // take the identical path.
        c.set(
            e,
            "Hazard",
            &Hazard { speed: START_SPEED, width: 1.0, height: 1.0, active: 0 },
        );
        park(c, e, i);
    }
}

/// The first entity carrying `name`, and its `T`. `T` need not be the named
/// component — `first::<Sprite>(c, "Body")` finds the runner, which is the
/// only thing with a `Body`, and hands back its `Sprite`.
fn first<T: Copy>(c: &mut Ctl, name: &'static str) -> Option<(Entity, T)> {
    let mut ids = [0u64; 4];
    if c.query(name, &mut ids) == 0 {
        return None;
    }
    let want = if name == "Body" { "Sprite" } else { name };
    c.get::<T>(ids[0], want).map(|v| (ids[0], v))
}

se::control! {
    name: "runner",
    slots: [],
    start: start,
    tick: tick,
}

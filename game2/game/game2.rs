//! The control layer: a runner, six cacti, and the ground.
//!
//! Its whole effect is the control var set. It asks for a roster by writing
//! the pool's length, seeds the rows once, and then only ever writes `view`,
//! `run` and `clock` — plus the two things a rule rather than a transform
//! decides: which parked cactus comes back out, and whether the runner has
//! hit one.
//!
//! Collision is here and not in a stage on purpose. A stage transforms rows;
//! whether the run is over is a rule about the game, and the game state it
//! sets is a control var, which only this module may write.

use crate::*;

const DT: f32 = 1.0 / 60.0;
/// Row 0 is the runner, 1..=6 are cacti, 7 is the ground.
const RUNNER: u32 = 0;
const HAZARDS: u32 = 6;
const GROUND_ROW: u32 = 7;
const ROWS: u32 = 8;

const GROUND: f32 = -1.4;
const KEY_SPACE: u32 = 1 << 26;
const KEY_UP: u32 = 1 << 30;
/// `w`, for a keyboard where space is awkward to reach from a test harness.
const KEY_W: u32 = 1 << 22;
/// Enter, not `r`: the IDE keeps `r` for run/stop even while a game is
/// playing, and a key the host has already spent is not one a game may bind.
const KEY_ENTER: u32 = 1 << 27;

#[no_mangle]
pub extern "C" fn tick(frame: u32) -> u32 {
    unsafe {
        let clock = &mut *(arena::CLOCK as *mut Clock);
        clock.frame = frame;
        clock.dt = DT;
        clock.t += DT;

        let view = &mut *(arena::VIEW as *mut View);
        // Framed on the action rather than on the origin: the ground is at
        // -1.4 and nothing interesting happens above +1. A viewport is wide,
        // so the box is too.
        view.half_w = 6.4;
        view.half_h = 2.1;
        view.cx = 0.0;
        view.cy = -0.55;

        let run = &mut *(arena::RUN as *mut Run);
        let input = &*(arena::INPUT as *const Input);
        (*se_pool_mut(arena::POOL_SPRITES)).len = ROWS;

        if clock.frame == 0 || (run.alive == 0 && input.pressed & KEY_ENTER != 0) {
            reset(run);
            return 0;
        }
        if run.alive == 0 {
            return 0;
        }

        let cs = se_col(arena::COL_SPRITE);
        let cb = se_col(arena::COL_BODY);
        let ch = se_col(arena::COL_HAZARD);

        // Jump. The only input the game takes, and it reads it from the arena
        // like everything else — no host call, no import.
        let body = &mut *se_row::<Body>(&cb, RUNNER);
        if input.pressed & (KEY_SPACE | KEY_UP | KEY_W) != 0 && body.on_ground == 1 {
            body.vel[1] = 9.5;
            body.on_ground = 0;
        }

        // Faster the longer you last.
        run.speed = 4.0 + (run.score as f32) * 0.02;
        run.spawn_t -= DT;
        if run.spawn_t <= 0.0 {
            run.spawn_t = 1.5 - (run.speed - 4.0).min(0.9) * 0.5;
            release(&cs, &ch, run.speed);
        }

        // Collision: a centre-distance test against half-extents, which is why
        // `pos` is the centre and not a corner.
        let r = *se_row::<Sprite>(&cs, RUNNER);
        for i in 1..=HAZARDS {
            let h = &*se_row::<Hazard>(&ch, i);
            if h.live != 1 {
                continue;
            }
            let s = &*se_row::<Sprite>(&cs, i);
            let dx = (r.pos[0] - s.pos[0]).abs();
            let dy = (r.pos[1] - s.pos[1]).abs();
            if dx < (r.size[0] + h.width) * 0.5 && dy < (r.size[1] + h.height) * 0.5 {
                run.alive = 0;
                return 0;
            }
        }
        run.score += 1;
        0
    }
}

/// Hand a parked cactus back out at the right edge.
unsafe fn release(cs: &Column, ch: &Column, speed: f32) {
    for i in 1..=HAZARDS {
        let h = &mut *se_row::<Hazard>(ch, i);
        if h.live == 1 {
            continue;
        }
        let tall = 0.55 + ((i * 37) % 5) as f32 * 0.12;
        *h = Hazard { speed, width: 0.3, height: tall, live: 1 };
        *se_row::<Sprite>(cs, i) = Sprite {
            pos: [6.6, GROUND + tall * 0.5],
            size: [0.34, tall],
            tint: [0.20, 0.52, 0.26, 1.0],
            uv: [0.5, 0.0, 1.0, 0.5],
        };
        return;
    }
}

/// The roster, from nothing. Idempotent, so a restart is the same call.
unsafe fn reset(run: &mut Run) {
    *run = Run { alive: 1, score: 0, speed: 4.0, spawn_t: 0.6 };

    let cs = se_col(arena::COL_SPRITE);
    let cb = se_col(arena::COL_BODY);
    let ch = se_col(arena::COL_HAZARD);

    *se_row::<Sprite>(&cs, RUNNER) = Sprite {
        pos: [-3.0, GROUND + 0.45],
        size: [0.6, 0.9],
        tint: [0.16, 0.18, 0.24, 1.0],
        uv: [0.0, 0.0, 0.5, 0.5],
    };
    *se_row::<Body>(&cb, RUNNER) = Body { vel: [0.0, 0.0], on_ground: 1, ..Body::ZERO };
    *se_row::<Hazard>(&ch, RUNNER) = Hazard::ZERO;

    for i in 1..=HAZARDS {
        *se_row::<Hazard>(&ch, i) = Hazard::ZERO;
        *se_row::<Body>(&cb, i) = Body { vel: [0.0, 0.0], on_ground: 2, ..Body::ZERO };
        *se_row::<Sprite>(&cs, i) = Sprite {
            pos: [-12.0, GROUND],
            size: [0.0, 0.0],
            tint: [0.20, 0.52, 0.26, 1.0],
            uv: [0.5, 0.0, 1.0, 0.5],
        };
    }

    // The ground is a sprite like everything else: one row, drawn by the same
    // call, and standing still because its body says it is scenery.
    *se_row::<Sprite>(&cs, GROUND_ROW) = Sprite {
        pos: [0.0, GROUND - 0.7],
        size: [40.0, 1.4],
        tint: [0.36, 0.30, 0.24, 1.0],
        uv: [0.0, 0.5, 0.5, 1.0],
    };
    *se_row::<Body>(&cb, GROUND_ROW) = Body { vel: [0.0, 0.0], on_ground: 2, ..Body::ZERO };
    *se_row::<Hazard>(&ch, GROUND_ROW) = Hazard::ZERO;
}

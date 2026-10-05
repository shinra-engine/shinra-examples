//! The control layer: a ship, a swarm of thirty-two bees, and the bullets and
//! bombs between them.
//!
//! The ship fires on its own, like a 雷電 — the player only steers. The swarm
//! sweeps sideways as one and drops a step each time it reaches a wall, faster
//! the fewer bees are left, and now and then a bee lets a bomb go. Shoot every
//! bee and the run is won; get bombed, or let the swarm reach the ship's row,
//! and it is lost. Enter starts again.
//!
//! The rules live here and the motion lives in `process/fly.rs`: whether a
//! bullet hit a bee is a rule about the game, and the state it changes is a
//! control var, which only this module may write.

use crate::*;

const DT: f32 = 1.0 / 60.0;

/// The pool: row 0 is the ship, then bullets, bombs, bees, stars, and a bar.
const SHIP: u32 = 0;
const BULLETS: u32 = 6;
const B0: u32 = 1;
const BOMBS: u32 = 6;
const O0: u32 = B0 + BULLETS;
const COLS: u32 = 8;
const BEE_ROWS: u32 = 4;
const BEES: u32 = COLS * BEE_ROWS;
const E0: u32 = O0 + BOMBS;
const STARS: u32 = 20;
const S0: u32 = E0 + BEES;
const BAR: u32 = S0 + STARS;
const ROWS: u32 = BAR + 1;

/// What a row is: the stage reads `kind`, the shader reads the shape id.
const K_SHIP: u32 = 1;
const K_BULLET: u32 = 2;
const K_BEE: u32 = 3;
const K_BOMB: u32 = 4;
const K_STAR: u32 = 5;
const SHAPE_SHIP: f32 = 0.0;
const SHAPE_BULLET: f32 = 1.0;
const SHAPE_BEE: f32 = 2.0;
const SHAPE_BOMB: f32 = 3.0;
const SHAPE_STAR: f32 = 4.0;
const SHAPE_BAR: f32 = 5.0;

const HALF_W: f32 = 4.0;
const HALF_H: f32 = 2.25;
/// How far the ship and the swarm may go sideways.
const EDGE: f32 = 3.6;
const SHIP_Y: f32 = -1.85;
const STEP_DOWN: f32 = 0.20;

const KEY_A: u32 = 1 << 0;
const KEY_D: u32 = 1 << 3;
/// Enter, not `r`: the IDE keeps `r` for run/stop even while a game is
/// playing, and a key the host has already spent is not one a game may bind.
const KEY_ENTER: u32 = 1 << 27;
const KEY_LEFT: u32 = 1 << 28;
const KEY_RIGHT: u32 = 1 << 29;

#[no_mangle]
pub extern "C" fn tick(frame: u32) -> u32 {
    unsafe {
        let clock = &mut *(arena::CLOCK as *mut Clock);
        clock.frame = frame;
        clock.dt = DT;
        clock.t += DT;

        let view = &mut *(arena::VIEW as *mut View);
        view.half_w = HALF_W;
        view.half_h = HALF_H;
        view.cx = 0.0;
        view.cy = 0.0;

        let run = &mut *(arena::RUN as *mut Run);
        let input = &*(arena::INPUT as *const Input);
        (*se_pool_mut(arena::POOL_SPRITES)).len = ROWS;

        if clock.frame == 0 || (run.alive != 1 && input.pressed & KEY_ENTER != 0) {
            reset(run);
            return 0;
        }
        if run.alive != 1 {
            return 0;
        }

        let cs = se_col(arena::COL_SPRITE);
        let ca = se_col(arena::COL_ACTOR);

        // Steer. The only input the game takes, read from the arena like
        // everything else — no host call, no import.
        let mut dx = 0.0;
        if input.down & (KEY_LEFT | KEY_A) != 0 {
            dx -= 1.0;
        }
        if input.down & (KEY_RIGHT | KEY_D) != 0 {
            dx += 1.0;
        }
        let ship = &mut *se_row::<Sprite>(&cs, SHIP);
        ship.pos[0] = (ship.pos[0] + dx * 4.2 * DT).clamp(-EDGE, EDGE);
        let ship_pos = ship.pos;

        // Fire, without being asked.
        run.fire_t -= DT;
        if run.fire_t <= 0.0 {
            run.fire_t = 0.24;
            launch(&cs, &ca, B0, BULLETS, K_BULLET, SHAPE_BULLET, ship_pos, [0.0, 6.5], [0.07, 0.30], [1.0, 0.92, 0.30, 1.0]);
        }

        // The swarm: one heading for all of them, reversed and lowered when
        // the outermost live bee reaches a wall.
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        for i in E0..E0 + BEES {
            if (*se_row::<Actor>(&ca, i)).live == 1 {
                let x = (*se_row::<Sprite>(&cs, i)).pos[0];
                min_x = min_x.min(x);
                max_x = max_x.max(x);
            }
        }
        if (run.dir > 0.0 && max_x > EDGE) || (run.dir < 0.0 && min_x < -EDGE) {
            run.dir = -run.dir;
            for i in E0..E0 + BEES {
                if (*se_row::<Actor>(&ca, i)).live == 1 {
                    (*se_row::<Sprite>(&cs, i)).pos[1] -= STEP_DOWN;
                }
            }
        }
        let speed = 0.55 + (BEES - run.bees) as f32 * 0.045;
        for i in E0..E0 + BEES {
            let a = &mut *se_row::<Actor>(&ca, i);
            if a.live == 1 {
                a.vel = [run.dir * speed, 0.0];
            }
        }

        // A bee lets a bomb go: the k-th live one, k from a seed that only
        // control advances, so a replay of the same inputs is the same run.
        run.bomb_t -= DT;
        if run.bomb_t <= 0.0 && run.bees > 0 {
            run.rng = run.rng.wrapping_mul(1664525).wrapping_add(1013904223);
            let mut k = (run.rng >> 8) % run.bees;
            for i in E0..E0 + BEES {
                if (*se_row::<Actor>(&ca, i)).live != 1 {
                    continue;
                }
                if k == 0 {
                    let at = (*se_row::<Sprite>(&cs, i)).pos;
                    launch(&cs, &ca, O0, BOMBS, K_BOMB, SHAPE_BOMB, at, [0.0, -1.7], [0.16, 0.16], [1.0, 0.30, 0.25, 1.0]);
                    break;
                }
                k -= 1;
            }
            run.bomb_t = (1.1 - (BEES - run.bees) as f32 * 0.02).max(0.35);
        }

        // A bullet that reaches a bee kills both.
        for b in B0..B0 + BULLETS {
            if (*se_row::<Actor>(&ca, b)).live != 1 {
                continue;
            }
            let sb = *se_row::<Sprite>(&cs, b);
            for e in E0..E0 + BEES {
                if (*se_row::<Actor>(&ca, e)).live != 1 {
                    continue;
                }
                if overlap(&sb, &*se_row::<Sprite>(&cs, e)) {
                    park(&cs, &ca, b);
                    park(&cs, &ca, e);
                    run.score += 10;
                    run.bees -= 1;
                    break;
                }
            }
        }

        // A bomb that reaches the ship ends the run.
        let ship = *se_row::<Sprite>(&cs, SHIP);
        for o in O0..O0 + BOMBS {
            if (*se_row::<Actor>(&ca, o)).live == 1 && overlap(&ship, &*se_row::<Sprite>(&cs, o)) {
                run.alive = 0;
            }
        }
        // So does a swarm that has come down to the ship's row.
        for e in E0..E0 + BEES {
            if (*se_row::<Actor>(&ca, e)).live == 1 && (*se_row::<Sprite>(&cs, e)).pos[1] < SHIP_Y + 0.35 {
                run.alive = 0;
            }
        }
        if run.bees == 0 && run.alive == 1 {
            run.alive = 2;
        }

        // The bar is how many bees are left, as a length.
        let bar = &mut *se_row::<Sprite>(&cs, BAR);
        bar.size = [3.0 * run.bees as f32 / BEES as f32, 0.08];
        bar.pos = [-HALF_W + 0.1 + bar.size[0] * 0.5, HALF_H - 0.12];
        0
    }
}

/// Centre-distance test against half-extents, which is why `pos` is the centre.
fn overlap(a: &Sprite, b: &Sprite) -> bool {
    (a.pos[0] - b.pos[0]).abs() < (a.size[0] + b.size[0]) * 0.5
        && (a.pos[1] - b.pos[1]).abs() < (a.size[1] + b.size[1]) * 0.5
}

/// Hand the first parked row in `first..first + n` out as something new.
unsafe fn launch(
    cs: &Column,
    ca: &Column,
    first: u32,
    n: u32,
    kind: u32,
    shape: f32,
    at: [f32; 2],
    vel: [f32; 2],
    size: [f32; 2],
    tint: [f32; 4],
) {
    for i in first..first + n {
        let a = &mut *se_row::<Actor>(ca, i);
        if a.live == 1 {
            continue;
        }
        *a = Actor { vel, kind, live: 1 };
        *se_row::<Sprite>(cs, i) = Sprite { pos: at, size, tint, uv: [shape, 0.0, 0.0, 0.0] };
        return;
    }
}

/// Take a row out of play: not moved, not tested, and nothing to draw.
unsafe fn park(cs: &Column, ca: &Column, i: u32) {
    (*se_row::<Actor>(ca, i)).live = 0;
    let s = &mut *se_row::<Sprite>(cs, i);
    s.size = [0.0, 0.0];
    s.pos = [0.0, -9.0];
}

/// The roster, from nothing. Idempotent, so a restart is the same call.
unsafe fn reset(run: &mut Run) {
    *run = Run { alive: 1, score: 0, dir: 1.0, fire_t: 0.3, bomb_t: 1.2, rng: 12345, bees: BEES };

    let cs = se_col(arena::COL_SPRITE);
    let ca = se_col(arena::COL_ACTOR);
    let parked = Sprite { pos: [0.0, -9.0], size: [0.0, 0.0], tint: [1.0, 1.0, 1.0, 1.0], uv: [0.0; 4] };

    for i in 0..ROWS {
        *se_row::<Actor>(&ca, i) = Actor::ZERO;
        *se_row::<Sprite>(&cs, i) = parked;
    }

    *se_row::<Sprite>(&cs, SHIP) = Sprite {
        pos: [0.0, SHIP_Y],
        size: [0.46, 0.40],
        tint: [0.30, 0.85, 1.0, 1.0],
        uv: [SHAPE_SHIP, 0.0, 0.0, 0.0],
    };
    *se_row::<Actor>(&ca, SHIP) = Actor { vel: [0.0, 0.0], kind: K_SHIP, live: 1 };

    // Four ranks of eight, each rank a colour.
    let ranks = [
        [0.95, 0.35, 0.80, 1.0],
        [1.0, 0.80, 0.20, 1.0],
        [1.0, 0.60, 0.15, 1.0],
        [0.55, 0.90, 0.35, 1.0],
    ];
    for r in 0..BEE_ROWS {
        for c in 0..COLS {
            let i = E0 + r * COLS + c;
            *se_row::<Sprite>(&cs, i) = Sprite {
                pos: [-1.75 + c as f32 * 0.5, 1.65 - r as f32 * 0.42],
                size: [0.40, 0.32],
                tint: ranks[r as usize],
                uv: [SHAPE_BEE, 0.0, 0.0, 0.0],
            };
            *se_row::<Actor>(&ca, i) = Actor { vel: [0.0, 0.0], kind: K_BEE, live: 1 };
        }
    }

    // Stars: a slow drift that gives the dark something to move against.
    let mut seed = 7u32;
    for k in 0..STARS {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let x = ((seed >> 8) % 1000) as f32 / 1000.0 * 2.0 * HALF_W - HALF_W;
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let y = ((seed >> 8) % 1000) as f32 / 1000.0 * 2.0 * HALF_H - HALF_H;
        let slow = 0.25 + (k % 4) as f32 * 0.18;
        *se_row::<Sprite>(&cs, S0 + k) = Sprite {
            pos: [x, y],
            size: [0.035, 0.035],
            tint: [0.75, 0.80, 1.0, 1.0],
            uv: [SHAPE_STAR, 0.0, 0.0, 0.0],
        };
        *se_row::<Actor>(&ca, S0 + k) = Actor { vel: [0.0, -slow], kind: K_STAR, live: 1 };
    }

    *se_row::<Sprite>(&cs, BAR) = Sprite {
        pos: [-HALF_W + 1.6, HALF_H - 0.12],
        size: [3.0, 0.08],
        tint: [0.95, 0.80, 0.25, 1.0],
        uv: [SHAPE_BAR, 0.0, 0.0, 0.0],
    };
}

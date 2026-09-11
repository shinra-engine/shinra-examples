//! The control layer: three planets going round, and a camera you can turn.
//!
//! Its entire effect on the world is the control var set `contract.wit`
//! declares. There is no spawn, no despawn, no `set`, no host call — so this
//! module imports nothing at all, which is what lets the engine run it on a
//! stack it shares with the data pipeline.
//!
//! Asking for a roster is `look.bodies = n`. The host reconciles the pool's
//! length against it; a body that needs seeding is seeded by a stage, because
//! the values are a pure function of the row index and that is the data
//! pipeline's business rather than control's.

use crate::*;

/// One sixtieth. The control layer is the only station that knows the time.
const DT: f32 = 1.0 / 60.0;

/// How many models `render/` can draw. The control layer counts them because
/// it is the one choosing; the shader switches on the number it is handed.
const MODELS: u32 = 3;

/// The keyboard is three bitmasks: bit 0..25 are a..z, so `m` is bit 12 and
/// `g` is bit 6. Spelled out here rather than imported, because a module
/// imports nothing at all.
const KEY_M: u32 = 1 << 12;
const KEY_G: u32 = 1 << 6;

#[no_mangle]
pub extern "C" fn tick(frame: u32) -> u32 {
    unsafe {
        let clock = &mut *(arena::CLOCK as *mut Clock);
        clock.frame = frame;
        clock.dt = DT;
        clock.t += DT;

        let input = &*(arena::INPUT as *const Input);
        let look = &mut *(arena::LOOK as *mut Look);

        // `m` cycles the model. The control layer's whole effect is this
        // number; what it means is the pipeline's business, and the host
        // never learns either.
        if input.pressed & KEY_M != 0 {
            look.asset = (look.asset + 1) % MODELS;
        }
        // `g` cycles the look, the same way.
        if input.pressed & KEY_G != 0 {
            look.graph = (look.graph + 1) % 2;
        }
        // Three bodies, and a fourth that arrives after a second — a roster
        // change with no spawn anywhere in sight. Writing the pool's length is
        // the whole of it; the host clamps the answer to the capacity it
        // placed and the pipeline picks it up on the same frame.
        look.bodies = if clock.t > 1.0 { 4 } else { 3 };

        let cam = &mut *(arena::CAMERA as *mut Camera);
        if clock.frame == 0 {
            cam.focus = [0.0, 0.0, 0.0];
            cam.dist = 4.2;
            cam.fov = 0.9;
            cam.pitch = 0.35;
        }
        cam.yaw += DT * 0.35;
        // The eye follows from the orbit the control layer keeps, so a graph
        // never has to recompute it.
        let (sy, cy) = (sinf(cam.yaw), cosf(cam.yaw));
        cam.eye = [
            cam.focus[0] + cy * cam.dist,
            cam.focus[1] + cam.pitch * cam.dist,
            cam.focus[2] + sy * cam.dist,
        ];

        (*se_pool_mut(arena::POOL_BODIES)).len = look.bodies;
        seed(look.bodies);
        0
    }
}

/// New rows arrive zeroed, and a body of zero size is not a body. Seeding is
/// a pure function of the row index, so it belongs to whoever asked for the
/// row — and it is idempotent, so running it every frame costs an `if`.
unsafe fn seed(n: u32) {
    let co = se_col(arena::COL_ORBIT);
    let cs = se_col(arena::COL_SPIN);
    let cb = se_col(arena::COL_BREATHE);
    for i in 0..n {
        let o = &mut *se_row::<Orbit>(&co, i);
        if o.radius != 0.0 {
            continue;
        }
        let t = i as f32 / n.max(1) as f32;
        *o = Orbit {
            center: [0.0, 0.0, 0.0],
            radius: 1.1 + (i % 3) as f32 * 0.55,
            speed: 0.55 + (i % 4) as f32 * 0.18,
            phase: t * 6.283_185_5,
            tilt: sinf(i as f32 * 0.37) * 0.6,
            ..Orbit::ZERO
        };
        *se_row::<Spin>(&cs, i) = Spin {
            axis: [0.2, 1.0, sinf(i as f32 * 0.7) * 0.4],
            rate: 0.8 + (i % 5) as f32 * 0.25,
            angle: 0.0,
            ..Spin::ZERO
        };
        *se_row::<Breathe>(&cb, i) = Breathe {
            base: 1.15,
            amount: 0.08,
            rate: 1.1 + (i % 3) as f32 * 0.3,
            phase: t * 6.28,
        };
    }
}

fn sinf(x: f32) -> f32 {
    const TAU: f32 = 6.283_185_5;
    const PI: f32 = 3.141_592_7;
    let mut a = x % TAU;
    if a < 0.0 {
        a += TAU;
    }
    let (a, sign) = if a > PI { (a - PI, -1.0) } else { (a, 1.0) };
    let n = 16.0 * a * (PI - a);
    sign * n / (5.0 * PI * PI - 4.0 * a * (PI - a))
}

fn cosf(x: f32) -> f32 {
    sinf(x + 1.570_796_4)
}

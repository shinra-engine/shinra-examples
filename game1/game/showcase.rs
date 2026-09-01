//! A second control for the same bundle: attract mode.
//!
//! `game/` holds alternatives, and this is the cheapest possible proof that
//! swapping one changes the *rules* while the contract, the stages and the
//! graph all stay exactly as they were. It ignores the keyboard and cycles the
//! themes on a timer instead.
//!
//! Run it with: `shinra run game1 --game showcase`

use data::{Breathe, Camera, Orbit, Spin, Transform};
use se::{Ctl, Frame, Slot};

const THEMES: [&str; 3] = ["bunny", "teapot", "quad"];
/// Seconds each theme is shown before the slot is repointed.
const DWELL: f64 = 4.0;

fn start(c: &mut Ctl) {
    let mut buf = [0u64; 64];
    if c.query("Orbit", &mut buf) == 0 {
        for i in 0..8u32 {
            let e = c.spawn();
            let t = i as f32 / 8.0;
            c.set(e, "Transform", &Transform::IDENTITY);
            c.set(
                e,
                "Orbit",
                &Orbit {
                    center: [0.0, 0.0, 0.0],
                    radius: 1.8 + (i % 4) as f32 * 0.9,
                    speed: 0.4 + t * 0.9,
                    phase: t * std::f32::consts::TAU,
                    tilt: t * 1.2 - 0.6,
                },
            );
            c.set(e, "Spin", &Spin { axis: [0.0, 1.0, 0.3], rate: 1.0 + t, angle: 0.0 });
            c.set(
                e,
                "Breathe",
                &Breathe { base: 0.5, amount: 0.12, rate: 0.9 + t, phase: t * 6.28 },
            );
        }
    }
    if c.query("Camera", &mut buf) == 0 {
        let e = c.spawn();
        c.set(
            e,
            "Camera",
            &Camera {
                eye: [0.0, 2.4, 10.0],
                focus: [0.0, 0.0, 0.0],
                fov: 0.85,
                dist: 10.0,
                yaw: 0.0,
                pitch: 0.24,
            },
        );
    }
    c.log("showcase: attract mode");
}

fn tick(c: &mut Ctl, f: &Frame) {
    // Which theme this instant belongs to, derived from the clock rather than
    // remembered — a module's statics do not survive its own hot swap, so
    // anything worth keeping is either data or a function of time.
    let slot = ((f.t / DWELL) as usize) % THEMES.len();
    let prev = (((f.t - f.dt as f64) / DWELL) as usize) % THEMES.len();
    if slot != prev || f.index == 1 {
        c.set_slot(Slot::Asset, THEMES[slot]);
    }

    let mut buf = [0u64; 4];
    if c.query("Camera", &mut buf) > 0 {
        if let Some(mut cam) = c.get::<Camera>(buf[0], "Camera") {
            cam.yaw = (f.t * 0.25) as f32;
            cam.pitch = 0.25 + ((f.t * 0.4).sin() * 0.18) as f32;
            let (sy, cy) = cam.yaw.sin_cos();
            let (sp, cp) = cam.pitch.sin_cos();
            cam.eye = [cam.dist * cp * sy, cam.dist * sp, cam.dist * cp * cy];
            c.set(buf[0], "Camera", &cam);
        }
    }
}

se::control! {
    name: "showcase",
    slots: [
        se::SlotBind::new(Slot::Asset, THEMES[0]),
        se::SlotBind::new(Slot::Render, "solid"),
    ],
    start: start,
    tick: tick,
}

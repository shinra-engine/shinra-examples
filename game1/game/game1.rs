//! Control for game1: the only station that knows the time.
//!
//! Its write surface is exactly what the design allows — the input components
//! (`Camera`, `Look`) and spawn/despawn. It never touches a `Transform`; the
//! stages in `process/` own those. What it does own is the decision of *which
//! module fills which slot*, which is how pressing M changes the model without
//! anything in `render/` or `process/` knowing it happened.

use data::{Breathe, Camera, Look, Orbit, Spin, Transform};
use se::{key, Ctl, Entity, Frame, Slot};

/// The asset modules this game can point its asset slot at. Adding a theme
/// means adding an `asset/*.rs` and a name here.
const THEMES: [&str; 3] = ["bunny", "teapot", "quad"];
/// The render modules it can point its render slot at.
const LOOKS: [&str; 2] = ["solid", "ghost"];

const MAX_BODIES: u32 = 12;
const MIN_BODIES: u32 = 1;

/// Called once the slots are bound, and again after any hot reload — so it
/// reconciles rather than spawns. What should exist is compared against what
/// does, and only the difference is acted on. Running it twice is a no-op,
/// which is what makes reloading `game1.so` safe mid-flight.
fn start(c: &mut Ctl) {
    let look = ensure_look(c);
    ensure_camera(c);
    reconcile_roster(c, look.bodies);
    c.log("game1: roster reconciled");
}

fn tick(c: &mut Ctl, f: &Frame) {
    let input = unsafe { f.input() };
    let Some((look_e, mut look)) = find_look(c) else { return };
    let mut changed = false;

    // --- slots: what the world is made of ------------------------------
    if input.just_pressed(b'M') {
        look.asset = (look.asset + 1) % THEMES.len() as u32;
        c.set_slot(Slot::Asset, THEMES[look.asset as usize]);
        changed = true;
    }
    if input.just_pressed(b'G') {
        look.graph = (look.graph + 1) % LOOKS.len() as u32;
        c.set_slot(Slot::Render, LOOKS[look.graph as usize]);
        changed = true;
    }

    // --- roster: what exists -------------------------------------------
    if input.just_pressed(b'=') || input.just_pressed(b'+') {
        look.bodies = (look.bodies + 1).min(MAX_BODIES);
        changed = true;
    }
    if input.just_pressed(b'-') {
        look.bodies = look.bodies.saturating_sub(1).max(MIN_BODIES);
        changed = true;
    }
    if changed {
        c.set(look_e, "Look", &look);
        reconcile_roster(c, look.bodies);
    }

    // --- the view: an input component, so control may write it ----------
    if let Some((cam_e, mut cam)) = find_camera(c) {
        let speed = 1.6 * f.dt;
        if input.is_down(key::LEFT) {
            cam.yaw -= speed;
        }
        if input.is_down(key::RIGHT) {
            cam.yaw += speed;
        }
        if input.is_down(key::UP) {
            cam.pitch = (cam.pitch + speed).min(1.3);
        }
        if input.is_down(key::DOWN) {
            cam.pitch = (cam.pitch - speed).max(-1.3);
        }
        if input.is_down(b'W') {
            cam.dist = (cam.dist - 6.0 * f.dt).max(2.0);
        }
        if input.is_down(b'S') {
            cam.dist = (cam.dist + 6.0 * f.dt).min(40.0);
        }
        // Drift when nobody is driving, so an idle screen still shows depth.
        if !touching(input) {
            cam.yaw += 0.12 * f.dt;
        }

        let (sy, cy) = cam.yaw.sin_cos();
        let (sp, cp) = cam.pitch.sin_cos();
        cam.eye = [
            cam.focus[0] + cam.dist * cp * sy,
            cam.focus[1] + cam.dist * sp,
            cam.focus[2] + cam.dist * cp * cy,
        ];
        c.set(cam_e, "Camera", &cam);
    }

    // GameTok: a swipe is not this game's business beyond passing it on.
    if input.swipe != 0 {
        c.swipe(input.swipe as i32);
    }
}

fn touching(i: &se::Input) -> bool {
    i.is_down(key::LEFT) || i.is_down(key::RIGHT) || i.is_down(key::UP) || i.is_down(key::DOWN)
}

// ---------------------------------------------------------------------------
// reconcile: what should exist, against what does
// ---------------------------------------------------------------------------

fn reconcile_roster(c: &mut Ctl, want: u32) {
    let mut have = [0u64; 64];
    let n = c.query("Orbit", &mut have).min(have.len() as u32);

    // Too few: add the missing ones, each with its own phase so they spread
    // around the ring instead of stacking up.
    for i in n..want {
        let e = c.spawn();
        let t = i as f32 / want.max(1) as f32;
        c.set(e, "Transform", &Transform::IDENTITY);
        c.set(
            e,
            "Orbit",
            &Orbit {
                center: [0.0, 0.0, 0.0],
                radius: 2.6 + (i % 3) as f32 * 1.0,
                speed: 0.55 + (i % 4) as f32 * 0.18,
                phase: t * std::f32::consts::TAU,
                tilt: (i as f32 * 0.37).sin() * 0.6,
            },
        );
        c.set(
            e,
            "Spin",
            &Spin {
                axis: [0.2, 1.0, (i as f32 * 0.7).sin() * 0.4],
                rate: 0.8 + (i % 5) as f32 * 0.25,
                angle: 0.0,
            },
        );
        c.set(
            e,
            "Breathe",
            &Breathe {
                base: 1.05,
                amount: 0.10,
                rate: 1.1 + (i % 3) as f32 * 0.3,
                phase: t * 6.28,
            },
        );
    }

    // Too many: drop from the end.
    for k in want..n {
        c.despawn(have[k as usize]);
    }
}

fn ensure_camera(c: &mut Ctl) -> Camera {
    if let Some((_, cam)) = find_camera(c) {
        return cam;
    }
    let cam = Camera {
        eye: [0.0, 2.0, 9.0],
        focus: [0.0, 0.0, 0.0],
        fov: 0.9,
        dist: 9.0,
        yaw: 0.0,
        pitch: 0.22,
    };
    let e = c.spawn();
    c.set(e, "Camera", &cam);
    cam
}

fn ensure_look(c: &mut Ctl) -> Look {
    if let Some((_, l)) = find_look(c) {
        return l;
    }
    let l = Look { asset: 0, graph: 0, bodies: 6 };
    let e = c.spawn();
    c.set(e, "Look", &l);
    l
}

fn find_camera(c: &mut Ctl) -> Option<(Entity, Camera)> {
    first(c, "Camera")
}
fn find_look(c: &mut Ctl) -> Option<(Entity, Look)> {
    first(c, "Look")
}

fn first<T: Copy>(c: &mut Ctl, name: &'static str) -> Option<(Entity, T)> {
    let mut buf = [0u64; 4];
    if c.query(name, &mut buf) == 0 {
        return None;
    }
    c.get::<T>(buf[0], name).map(|v| (buf[0], v))
}

se::control! {
    name: "orbit",
    slots: [
        se::SlotBind::new(Slot::Asset, THEMES[0]),
        se::SlotBind::new(Slot::Render, LOOKS[0]),
    ],
    start: start,
    tick: tick,
}

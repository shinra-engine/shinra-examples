use gametok_abi::{Drawable, InputFrame};
use std::cell::RefCell;

const MESH_PATHS: &[&str] = &["assets/bunny.obj"];

struct Player {
    pos: glam::Vec3,
    yaw: f32,
    pitch: f32,
    scale: f32,
}

thread_local! {
    static WORLD: RefCell<hecs::World> = RefCell::new(hecs::World::new());
    static FRAME: RefCell<Vec<Drawable>> = RefCell::new(Vec::new());
    static INIT: RefCell<bool> = const { RefCell::new(false) };
}

const MOVE_SPEED: f32 = 2.0;
const ROT_SPEED: f32 = 1.5;
const SCALE_RATE: f32 = 1.5;
const SCALE_MIN: f32 = 0.5;
const SCALE_MAX: f32 = 50.0;
const INITIAL_SCALE: f32 = 10.0;

fn ensure_init() {
    INIT.with(|i| {
        if !*i.borrow() {
            WORLD.with(|w| {
                w.borrow_mut().spawn((Player {
                    pos: glam::Vec3::ZERO,
                    yaw: 0.0,
                    pitch: 0.4,
                    scale: INITIAL_SCALE,
                },));
            });
            *i.borrow_mut() = true;
        }
    });
}

#[no_mangle]
pub extern "C" fn meshes_count() -> u32 {
    MESH_PATHS.len() as u32
}

#[no_mangle]
pub extern "C" fn meshes_path(i: u32, out: *mut u8, cap: u32) -> u32 {
    let path = match MESH_PATHS.get(i as usize) {
        Some(p) => *p,
        None => return 0,
    };
    let bytes = path.as_bytes();
    let n = bytes.len().min(cap as usize);
    if !out.is_null() && n > 0 {
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, n);
        }
    }
    n as u32
}

#[no_mangle]
pub extern "C" fn tick(dt: f32, input: *const InputFrame) {
    ensure_init();
    let inp = unsafe { input.as_ref().copied() }.unwrap_or_default();

    WORLD.with(|w| {
        let mut w = w.borrow_mut();
        for (_, p) in w.query_mut::<&mut Player>() {
            p.pos.x += inp.move_x * MOVE_SPEED * dt;
            p.pos.z += inp.move_z * MOVE_SPEED * dt;
            p.yaw += inp.rot_yaw * ROT_SPEED * dt;
            p.pitch += inp.rot_pitch * ROT_SPEED * dt;

            if inp.scale_delta != 0.0 {
                p.scale =
                    (p.scale * (SCALE_RATE.powf(inp.scale_delta * dt))).clamp(SCALE_MIN, SCALE_MAX);
            }
        }
    });

    FRAME.with(|f| {
        let mut buf = f.borrow_mut();
        buf.clear();
        WORLD.with(|w| {
            let w = w.borrow();
            for (_, p) in w.query::<&Player>().iter() {
                let m = glam::Mat4::from_scale_rotation_translation(
                    glam::Vec3::splat(p.scale),
                    glam::Quat::from_euler(glam::EulerRot::YXZ, p.yaw, p.pitch, 0.0),
                    p.pos,
                );
                buf.push(Drawable {
                    mesh_id: 0,
                    _pad: 0,
                    model: m.to_cols_array(),
                });
            }
        });
    });
}

#[no_mangle]
pub extern "C" fn drawables_ptr() -> *const Drawable {
    FRAME.with(|f| f.borrow().as_ptr())
}

#[no_mangle]
pub extern "C" fn drawables_len() -> u32 {
    FRAME.with(|f| f.borrow().len() as u32)
}

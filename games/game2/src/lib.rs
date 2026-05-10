#![allow(unused_variables, unused_mut, dead_code, unused_imports, unused_macros)]
#![allow(non_snake_case)]

use gametok_abi::Drawable;

include!(concat!(env!("OUT_DIR"), "/main.rs"));

const MESH_PATHS: &[&str] = &["assets/teapot.obj"];

thread_local! {
    static FRAME: RefCell<Vec<Drawable>> = RefCell::new(Vec::new());
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
    let inp = unsafe { input.as_ref().copied() }.unwrap_or_default();
    main_tick(dt, inp);

    FRAME.with(|f| {
        let mut buf = f.borrow_mut();
        buf.clear();
        query1::<Player, _>(|p| {
            let m = glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::splat(p.scale),
                glam::Quat::from_euler(glam::EulerRot::YXZ, p.yaw, p.pitch, 0.0),
                glam::Vec3::new(p.x, p.y, p.z),
            );
            buf.push(Drawable {
                mesh_id: 0,
                _pad: 0,
                model: m.to_cols_array(),
            });
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

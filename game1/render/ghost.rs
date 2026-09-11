//! The ghost look: the same bodies, flat and pale, one per body.
//!
//! A render module does not call WebGPU. It *records* — a run of fixed-size
//! commands into the arena — and the host replays them. One boundary crossing
//! a frame instead of one per draw, and this module keeps the zero-import
//! property the shared memory model depends on.
//!
//! Note what the draw carries: a column *index*, not a pointer and not a copy.
//! The host resolves it through the same descriptor the stages used, so the
//! instance buffer it uploads is the column's own bytes at the column's own
//! stride, with nothing repacked.

use crate::*;

/// The shader rides in a custom section. The host reads it out of the file
/// without instantiating anything, exactly as it reads an asset.
#[used]
#[link_section = "se.wgsl"]
static WGSL: [u8; include_bytes!("ghost/shader.wgsl").len()] =
    *include_bytes!("ghost/shader.wgsl");

const BEGIN_PASS: u32 = 0;
const DRAW_INSTANCED: u32 = 1;
const END_PASS: u32 = 3;

const BLANK: Command = Command {
    kind: BEGIN_PASS,
    vertices: 0,
    shader: 0,
    column: u32::MAX,
    count: 0,
    uniform_off: 0,
    uniform_len: 0,
    pad: 0,
    clear_r: 0.0,
    clear_g: 0.0,
    clear_b: 0.0,
    clear_a: 1.0,
};

#[no_mangle]
pub extern "C" fn record() -> u32 {
    unsafe {
        let bodies = se_pool(arena::POOL_BODIES).len;
        let cmds = arena::COMMANDS as *mut Command;

        *cmds.add(0) = Command {
            kind: BEGIN_PASS,
            clear_r: 0.10,
            clear_g: 0.02,
            clear_b: 0.14,
            ..BLANK
        };
        *cmds.add(1) = Command {
            kind: DRAW_INSTANCED,
            // Four triangles per body. The host draws what the module says;
            // it has no idea what the geometry is.
            vertices: 12,
            column: arena::COL_TRANSFORM,
            count: bodies,
            uniform_off: arena::CAMERA,
            uniform_len: Camera::SIZE,
            ..BLANK
        };
        *cmds.add(2) = Command { kind: END_PASS, ..BLANK };

        let f = &mut *(arena::FRAME as *mut Frame);
        f.commands = arena::COMMANDS;
        f.count = 3;
        f.present = 0;
        f.epoch = f.epoch.wrapping_add(1);
        3
    }
}

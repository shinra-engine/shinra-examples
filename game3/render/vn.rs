//! The visual-novel look: one instanced draw over every sprite in the pool.

use crate::render::*;
use crate::*;

#[used]
#[link_section = "se.wgsl"]
static WGSL: [u8; include_bytes!("vn/shader.wgsl").len()] =
    *include_bytes!("vn/shader.wgsl");


#[no_mangle]
pub extern "C" fn record() -> u32 {
    unsafe {
        let n = se_pool(arena::POOL_SPRITES).len;
        let cmds = arena::COMMANDS as *mut Command;

        *cmds.add(0) = Command {
            kind: BEGIN_PASS,
            clear_r: 0.03,
            clear_g: 0.04,
            clear_b: 0.07,
            ..BLANK
        };
        *cmds.add(1) = Command {
            kind: DRAW_INSTANCED,
            vertices: 6,
            column: arena::COL_SPRITE,
            count: n,
            uniform_off: arena::VIEW,
            uniform_len: View::SIZE,
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

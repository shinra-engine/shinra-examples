//! The swarm's look: one instanced draw over every sprite in the pool.
//!
//! The ship, the bees, the bullets, the bombs, the stars and the bar are all
//! the same column, so they are the same call. What each one looks like is a
//! shape id the shader reads out of `uv.x`; the clear colour carries the
//! state of the run — dark blue while playing, red once lost, green once won.

use crate::render::*;
use crate::*;

#[used]
#[link_section = "se.wgsl"]
static WGSL: [u8; include_bytes!("swarm/shader.wgsl").len()] =
    *include_bytes!("swarm/shader.wgsl");


#[no_mangle]
pub extern "C" fn record() -> u32 {
    unsafe {
        let n = se_pool(arena::POOL_SPRITES).len;
        let run = &*(arena::RUN as *const Run);
        let cmds = arena::COMMANDS as *mut Command;

        let (r, g, b) = match run.alive {
            1 => (0.02, 0.03, 0.09),
            2 => (0.02, 0.11, 0.06),
            _ => (0.14, 0.02, 0.03),
        };
        *cmds.add(0) = Command { kind: BEGIN_PASS, clear_r: r, clear_g: g, clear_b: b, ..BLANK };
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

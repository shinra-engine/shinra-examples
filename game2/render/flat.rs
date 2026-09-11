//! The flat look: one instanced draw over every sprite in the pool.
//!
//! One draw, not one per kind. The runner, the cacti and the ground are the
//! same column, so they are the same call — which is the reason they share a
//! pool, and the reason `sprite` is the record the contract makes widest.

use crate::*;

#[used]
#[link_section = "se.wgsl"]
static WGSL: [u8; include_bytes!("flat/shader.wgsl").len()] =
    *include_bytes!("flat/shader.wgsl");

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
        let n = se_pool(arena::POOL_SPRITES).len;
        let run = &*(arena::RUN as *const Run);
        let cmds = arena::COMMANDS as *mut Command;

        // The sky is the clear colour, and it dims when the run ends. A whole
        // pass for a gradient would be one more pipeline for one triangle.
        let k = if run.alive == 1 { 1.0 } else { 0.45 };
        *cmds.add(0) = Command {
            kind: BEGIN_PASS,
            clear_r: 0.86 * k,
            clear_g: 0.90 * k,
            clear_b: 0.95 * k,
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

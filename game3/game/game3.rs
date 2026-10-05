//! The control layer: reads the act from its blob and drives the cursor.
//!
//! Only this module writes control vars and the pool length. Nothing is kept
//! in statics, because a swap re-instantiates them: `Stage.act` and `Line` are
//! the state, and the script is parsed again every tick.

use crate::*;

mod cast;
mod parse;
mod reader;

const DT: f32 = 1.0 / 60.0;
// Host input bits (se-vm/src/input.rs). A module imports nothing, so spell them.
const SPACE: u32 = 1 << 26;
const ENTER: u32 = 1 << 27;
const KEY_A: u32 = 1;
/// Acts are looked at in this many blob slots.
const ACT_SLOTS: u32 = 8;

/// The bytes of act `act`. Blob slot == act index == sorted asset file order,
/// and sorting is by file name, so `act10` comes before `act2`.
unsafe fn story(act: u32) -> &'static [u8] {
    let s = se_blob(act);
    if act >= arena::BLOB_SLOTS || s.len == 0 {
        &[]
    } else {
        core::slice::from_raw_parts(s.off as *const u8, s.len as usize)
    }
}

/// How many of the first slots hold a script; at least 1 so the swap wraps.
unsafe fn acts() -> u32 {
    let mut n = 0;
    for i in 0..ACT_SLOTS.min(arena::BLOB_SLOTS) {
        if se_blob(i).len != 0 {
            n += 1;
        }
    }
    n.max(1)
}

#[no_mangle]
pub extern "C" fn tick(frame: u32) -> u32 {
    unsafe {
        let clock = &mut *(arena::CLOCK as *mut Clock);
        clock.frame = frame;
        clock.dt = DT;
        clock.t += DT;

        let view = &mut *(arena::VIEW as *mut View);
        view.half_w = 1.6;
        view.half_h = 0.9;
        view.cx = 0.0;
        view.cy = 0.0;

        let input = &*(arena::INPUT as *const Input);
        let stage = &mut *(arena::STAGE as *mut Stage);
        stage.fade = 1.0;
        let n = acts();
        let mut changed = false;
        if input.pressed & KEY_A != 0 {
            stage.act = (stage.act + 1) % n;
            changed = true;
        } else if stage.act >= n {
            stage.act = 0;
            changed = true;
        }

        let script = parse::Script::new(story(stage.act));
        let line = &mut *(arena::LINE as *mut Line);
        let mut c = reader::Cursor {
            index: line.index,
            revealed: line.revealed,
            speaker: line.speaker,
            done: line.done,
            len: line.len,
            elapsed: line.elapsed,
        };
        if frame == 0 || changed {
            reader::restart(&mut c, &script);
        } else {
            let advance = input.pressed & (SPACE | ENTER) != 0;
            reader::step(&mut c, clock.dt, advance, &script);
        }
        line.index = c.index;
        line.revealed = c.revealed;
        line.speaker = c.speaker;
        line.done = c.done;
        line.len = c.len;
        line.elapsed = c.elapsed;

        cast::write(script.cast());
        0
    }
}

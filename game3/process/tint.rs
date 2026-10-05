//! The only stage: the speaker is lit, everyone else dimmed, and the stage
//! fade multiplies through both. Only actor rows (`kind == 0`) are touched.

use crate::*;

const DIM: f32 = 0.45;

#[se::stage]
fn tint(s: &mut Sprite) {
    if s.kind != 0 {
        return;
    }
    let (line, st) = unsafe {
        (&*(arena::LINE as *const Line), &*(arena::STAGE as *const Stage))
    };
    let lit = if s.uv[0] as u32 == line.speaker { 1.0 } else { DIM };
    let v = lit * st.fade;
    s.tint = [v, v, v, st.fade];
}

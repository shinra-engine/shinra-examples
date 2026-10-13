//! Which mesh scale each body uses.
//!
//! One line of work, and it is the whole of "press m to change the model":
//! the control layer picks an index, this copies it onto every row, and the
//! graph's instance input carries it to the shader, which only uses it to scale
//! the mesh. The mesh itself is chosen by the render module's draw command.

use crate::*;

#[se::stage]
fn model(t: &mut Transform) {
    t.kind = unsafe { (*(arena::LOOK as *const Look)).asset };
}

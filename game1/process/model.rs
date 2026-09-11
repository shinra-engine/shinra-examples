//! What each body is drawn as.
//!
//! One line of work, and it is the whole of "press m to change the model":
//! the control layer picks an index, this copies it onto every row, and the
//! graph's instance input carries it to the shader. No module is rebuilt, no
//! asset is reloaded, and the pipeline does not learn what the index means.

use crate::*;

#[se::stage]
fn model(t: &mut Transform) {
    t.kind = unsafe { (*(arena::LOOK as *const Look)).asset };
}

//! The bunny theme.
//!
//! An asset module is name → bytes and nothing else. It cannot spawn the thing
//! it describes and cannot write a component — content never writes data. What
//! it can do is answer to the *same* name as its siblings: `bunny.so`,
//! `teapot.so` and `quad.so` all publish `model.obj`, so the render graph asks
//! for one mesh and the control layer decides which module is answering.

se::assets! {
    "model.obj" => include_bytes!("bunny/model.obj"),
}

//! The teapot theme. Same contract as `bunny.rs`, different bytes.
//!
//! Swapping this in is a slot change, not a reload of the game: the world
//! keeps every entity, every orbit phase and every spin angle, and only the
//! mesh under them changes.

se::assets! {
    "model.obj" => include_bytes!("teapot/model.obj"),
}

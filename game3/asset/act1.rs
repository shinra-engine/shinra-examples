//! Act one, as bytes.
//!
//! An asset module is name → bytes and nothing else. This one carries no mesh
//! and no texture — it carries the *story*. That is the point of game3: the
//! script is content, so replacing the content replaces the game, and
//! `process/`, `render/` and `game/` are not rebuilt and do not know which act
//! they are showing.
//!
//! The name is deliberately generic. Exactly as `game1/asset/bunny.rs` and
//! `teapot.rs` both publish `model.obj`, `act1.rs` and its siblings all
//! publish `story.rpy`, so the control layer asks for one script and the asset
//! slot decides who answers.
//!
//! Nothing here parses anything. Bytes cross the boundary; the dialect is read
//! on the far side, inside this bundle, never in an `se-*` crate. See
//! `act1/story.rpy` for the dialect — it is documented at the top of the file
//! it belongs to, next to the only lines that have to obey it.

se::assets! {
    "story.rpy" => include_bytes!("act1/story.rpy"),
}

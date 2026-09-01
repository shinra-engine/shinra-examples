//! Act two, as bytes.
//!
//! The same contract as `act1.rs`, a different story. Nothing is shared with
//! act one but the *name*: both modules publish `story.rpy`, so the control
//! layer asks for one script and the asset slot decides who answers. Pointing
//! the slot here changes the backdrop, the cast and every line without
//! rebuilding `bundle/`, `process/`, `render/` or `game/` — that is the claim
//! `design.md` makes about `asset.so`, and two modules under one name are what
//! make it checkable rather than merely stated.
//!
//! Act two has a four-name cast where act one has three. That costs one more
//! spawn and not one line elsewhere: an actor name in the script is a
//! *definition*, and the reconcile in `game/` is what turns definitions into
//! entities. Content never writes data.
//!
//! Nothing here parses anything. Bytes cross the boundary; the dialect is read
//! on the far side, inside this bundle, never in an `se-*` crate. See
//! `act2/story.rpy` — the dialect is documented once, at the top of
//! `act1/story.rpy`, next to the first lines that had to obey it.

se::assets! {
    "story.rpy" => include_bytes!("act2/story.rpy"),
}

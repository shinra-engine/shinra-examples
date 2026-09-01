//! game2's content: how the runner, the cacti and the ground look.
//!
//! An asset module is name → bytes and nothing else. It cannot spawn the
//! thing it describes and cannot write a component — content never writes
//! data. What it publishes here is not a mesh: game2 is flat, and a flat game
//! draws rectangles, so `render/flat.rs` names the empty mesh `""` and gets
//! the engine's built-in unit quad. Shipping a four-vertex `.obj` to say the
//! same thing would only add a file that can disagree with the built-in.
//!
//! So the bytes are `palette.txt`, the theme of the run: one linear RGBA per
//! role, read by `game/game2.rs` through `Ctl::asset` when it spawns or
//! reconciles the roster and written into each entity's `Sprite.tint`. That is
//! the whole point of the split — a sibling module publishing the same
//! `palette.txt` with darker numbers is night mode, a slot change that keeps
//! every entity, the score and the runner's height mid-jump, because it
//! changed the appearance and not one rule.
//!
//! Nothing in this file is a rule. There is no gravity, no jump impulse, no
//! scroll speed and no hitbox here; those are numbers in `process/` and
//! `game/`. If a value in `palette.txt` ever decides an outcome, it is in the
//! wrong file.
//!
//! `sprites/` is private to this module because it is named after it, so the
//! palette cannot be reached — or accidentally shared — by anything else.

se::assets! {
    "palette.txt" => include_bytes!("sprites/palette.txt"),
}

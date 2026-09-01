//! The component half of game2's contract.
//!
//! game2 is the no-network dinosaur: a 2D side-scroller. Everything the host
//! will ever store for it is declared here. `process/`, `render/` and `game/`
//! all import these names as `use data::...`, so this file is the whole
//! vocabulary of the game — and the only file whose change means "different
//! game set" rather than "reload a module".
//!
//! Nothing here is 3D. There is no `Transform`: a flat game needs a position
//! and a size, and the ortho camera is a component like any other.

/// One drawable rectangle. This is the instance component the flat pass draws,
/// so every field crosses into WGSL and every field is `f32`.
///
/// `pos` is the centre in world units, not a corner — collision in
/// `process/collide.rs` is then a centre-distance test against `size * 0.5`,
/// with no origin convention to get wrong.
#[repr(C)]
#[derive(Clone, Copy, Debug, se::Schema)]
pub struct Sprite {
    /// Centre, world units.
    pub pos: [f32; 2],
    /// Full width and height, world units.
    pub size: [f32; 2],
    /// Linear RGBA.
    pub tint: [f32; 4],
}

/// Vertical state of anything that falls. Only the runner has one.
///
/// `on_ground` is `u32` rather than `bool` because a layout is described in
/// scalars the host understands; it reads as a flag, 0 or 1. `_pad` keeps the
/// struct a whole number of 4-float rows so the packing is obvious on sight.
#[repr(C)]
#[derive(Clone, Copy, Debug, se::Schema)]
pub struct Body {
    /// World units per second.
    pub vel: [f32; 2],
    /// 1 while standing, 0 while airborne. `process/gravity.rs` owns it.
    pub on_ground: u32,
    pub _pad: u32,
}

/// A cactus. `process/scroll.rs` moves it leftwards at `speed`; `width` and
/// `height` are its collision box, which is deliberately its own number
/// rather than the `Sprite.size` it is drawn at — a hitbox may be forgiving.
///
/// `active` is how an obstacle is recycled instead of despawned: a stage
/// cannot spawn, so the roster stays fixed and off-screen cacti are parked
/// with `active == 0` until control hands one back out.
#[repr(C)]
#[derive(Clone, Copy, Debug, se::Schema)]
pub struct Hazard {
    /// Leftward world units per second.
    pub speed: f32,
    pub width: f32,
    pub height: f32,
    /// 1 when in play, 0 when parked.
    pub active: u32,
}

/// The view. A render pass names this component as its uniform, which is how
/// an orthographic camera reaches a shader without the render side being
/// given the world — the engine has no camera concept of its own.
///
/// Half-extents rather than left/right/top/bottom: `se_ortho` wants a box
/// around a centre, and `half` names are safe where `filter` and `target`
/// are not.
#[repr(C)]
#[derive(Clone, Copy, Debug, se::Schema)]
pub struct View {
    /// Half-width of the visible box, world units.
    pub half_w: f32,
    /// Half-height of the visible box, world units.
    pub half_h: f32,
    /// Centre of the visible box.
    pub cx: f32,
    pub cy: f32,
}

/// The run in progress: alive or crashed, how far, how fast, and when the
/// next cactus is due.
///
/// This lives in the world rather than in a `static` inside `game/game2.rs`
/// on purpose. Swapping a module drops its image and every `static` in it, so
/// state that must outlive a hot swap has to be data — and a game-over that
/// forgets itself on reload is the bug this rule exists to prevent.
#[repr(C)]
#[derive(Clone, Copy, Debug, se::Schema)]
pub struct Run {
    /// 1 while playing, 0 once the runner has hit something.
    pub alive: u32,
    pub score: u32,
    /// Current scroll speed; it ramps as the score climbs.
    pub speed: f32,
    /// Seconds until the next hazard is released.
    pub spawn_t: f32,
}

se::layouts!(Sprite, Body, Hazard, View, Run);

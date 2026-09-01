//! The component half of game1's contract.
//!
//! Everything the host will ever store for this game is declared here. The
//! host has no Rust types — it works from these layouts — so this file is what
//! `process/`, `render/` and `game/` all agree about, and the only file whose
//! change means "different game set" rather than "reload a module".

#[repr(C)]
#[derive(Clone, Copy, Debug, se::Schema)]
pub struct Transform {
    pub pos: [f32; 3],
    /// Unit quaternion, xyzw.
    pub rot: [f32; 4],
    pub scale: [f32; 3],
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        pos: [0.0; 3],
        rot: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0; 3],
    };
}

/// What makes a thing go round the middle. `process/orbit.rs` integrates
/// `phase` and writes the result into `Transform.pos`; nothing else may.
#[repr(C)]
#[derive(Clone, Copy, Debug, se::Schema)]
pub struct Orbit {
    pub center: [f32; 3],
    pub radius: f32,
    /// Radians per second.
    pub speed: f32,
    pub phase: f32,
    /// Tilt of the orbital plane, radians.
    pub tilt: f32,
}

/// Rotation about the body's own axis, independent of where it is.
#[repr(C)]
#[derive(Clone, Copy, Debug, se::Schema)]
pub struct Spin {
    pub axis: [f32; 3],
    pub rate: f32,
    pub angle: f32,
}

/// A slow scale wobble, so a still frame still reads as alive.
#[repr(C)]
#[derive(Clone, Copy, Debug, se::Schema)]
pub struct Breathe {
    pub base: f32,
    pub amount: f32,
    pub rate: f32,
    pub phase: f32,
}

/// The view. A render pass names this component as its uniform, which is how
/// a camera reaches a shader without the render side being given the world.
///
/// Fields are plain scalars: the host repacks them into the alignment WGSL
/// wants, so nothing here needs hand-written padding.
#[repr(C)]
#[derive(Clone, Copy, Debug, se::Schema)]
pub struct Camera {
    pub eye: [f32; 3],
    pub focus: [f32; 3],
    /// Vertical field of view, radians.
    pub fov: f32,
    /// Distance from `focus`, so the control layer can dolly without
    /// recomputing `eye` from scratch every tick.
    pub dist: f32,
    pub yaw: f32,
    pub pitch: f32,
}

/// What the player has chosen to look at.
///
/// This lives in the world rather than in a `static` inside `game/game1.rs`
/// on purpose: swapping a module drops its image and everything in it, so any
/// state that must outlive a hot swap has to be data. It is written only by
/// the control layer, and only in response to input.
#[repr(C)]
#[derive(Clone, Copy, Debug, se::Schema)]
pub struct Look {
    /// Index into the control layer's asset-module list.
    pub asset: u32,
    /// Index into its render-module list.
    pub graph: u32,
    /// How many bodies the roster should contain.
    pub bodies: u32,
}

se::layouts!(Transform, Orbit, Spin, Breathe, Camera, Look);

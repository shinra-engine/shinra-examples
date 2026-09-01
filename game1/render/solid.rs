//! The plain look: one instanced mesh pass, straight to the screen.
//!
//! Nodes and edges are all this module registers. It cannot spawn, cannot
//! write a component, and knows no time beyond the globals the host injects.
//! Swapping this file for `ghost.rs` therefore changes how the game looks and
//! is incapable of changing anything else — which is the point of the split.
//!
//! The shader is WGSL in `.wgsl` files, not strings in here. `render/wgsl/`
//! has no sibling `.rs`, so it is shared across the category; `render/solid/`
//! is named after this file, so it is private to it.

se::graph!("solid", |g| {
    g.present("scene").pass("bodies", |p| {
        p.shader(concat!(
            include_str!("wgsl/vs.wgsl"),
            include_str!("wgsl/light.wgsl"),
            include_str!("solid/fs.wgsl"),
        ))
        .color(&["scene"])
        .depth("depth")
        // A component, not a magic camera: the host uploads the first entity
        // carrying `Camera` as this pass's uniform.
        .uniform_of("Camera")
        .clear([0.05, 0.06, 0.10, 1.0])
        // `model.obj` names an asset, not a file. Whichever `asset/*.so` fills
        // the asset slot supplies it, so this module never learns whether it
        // drew a bunny or a teapot.
        .instanced("Transform", "model.obj")
    })
});

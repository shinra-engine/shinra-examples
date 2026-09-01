//! The trailing look: the same bodies, over a decaying image of themselves.
//!
//! This is the "view buffer as texture" rule doing real work. The `trail` pass
//! reads `trail` while writing it, which would be a cycle in any graph that
//! resolved reads within the frame. Here a sampled buffer always reads the
//! *previous* frame, so it is not a cycle at all — it is a one-frame feedback
//! loop.
//!
//! Two passes, so each names its own fragment stage; the vertex stage comes
//! from the category's shared `render/wgsl/`.

se::graph!("ghost", |g| {
    g.present("trail")
        .pass("bodies", |p| {
            p.shader(concat!(
                include_str!("wgsl/vs.wgsl"),
                include_str!("wgsl/light.wgsl"),
                include_str!("ghost/bodies.fs.wgsl"),
            ))
            .color(&["scene"])
            .depth("depth")
            .uniform_of("Camera")
            .clear([0.0, 0.0, 0.0, 1.0])
            .instanced("Transform", "model.obj")
        })
        .pass("trail", |p| {
            p.shader(include_str!("ghost/trail.fs.wgsl"))
                .color(&["trail"])
                .reads(&["scene", "trail"])
                .clear([0.0, 0.0, 0.0, 1.0])
        })
        // Ordering only — the read above needs no edge, because it resolves to
        // last frame no matter when this pass runs.
        .edge("bodies", "trail")
});

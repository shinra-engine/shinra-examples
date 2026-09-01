//! game3's look: a backdrop, the cast, and the box the dialogue goes in.
//!
//! Three layers, drawn back to front, and pass order *is* the layering. There
//! is no depth attachment: a visual novel is a flat stage, and `load` is one
//! flag covering colour and depth alike, so a pass that blends over the one
//! before it would be loading a depth buffer nobody cleared. Declaring the
//! order and leaving depth alone says the same thing with less machinery.
//!
//! ```text
//!   backdrop   fullscreen, uniform `Stage`   clears `scene`
//!   actors     instanced `Actor`, unit quad, uniform `View`   load
//!   box        fullscreen, uniform `View`                     load
//!   text       instanced `Glyph`, unit quad, uniform `View`   load
//! ```
//!
//! A pass gets one uniform, which is what splits them this way: the backdrop
//! needs `Stage.fade` and the drift, the other two need the view rectangle to
//! project into. `Actor.tint` is how the fade and "who is speaking" reach the
//! cast — the control layer multiplies both into it, exactly as the field's
//! doc comment in `bundle/data.rs` says — so the actors pass never needs to
//! see `Stage` at all.
//!
//! # Text
//!
//! The engine has no font, and no path from asset bytes to a sampled texture:
//! `bundle/buffer.rs` is the only thing that makes a texture, and `asset/*.so`
//! bytes arrive as `.obj` meshes. So an atlas is not expressible and a glyph
//! is *shaped* instead — one instanced unit quad per character, its bitmap a
//! constant table in `vn/font.wgsl`, its character code arriving in
//! `Glyph.uv.x`. No font asset, and nothing asked of the engine.
//!
//! The `text` pass therefore looks exactly like `actors`: same quad, same
//! `View`, a different fragment stage. It is last because it sits inside the
//! panel `box` draws, and it knows nothing about `Line` — the control layer
//! spawns only the revealed prefix, so the typewriter is a roster and not a
//! uniform this pass has to be told about.
//!
//! # What this module is not
//!
//! It is incapable of moving anything. `render/*.so` registers nodes and
//! edges; it cannot spawn, cannot write a component, and knows no time beyond
//! the globals the host injects. Swapping this file changes how game3 looks
//! and is unable to change anything else.
//!
//! WGSL lives in `.wgsl` files. `render/vn/` is named after this file, so it
//! is private to it — no other render module for game3 can reach this palette.

se::graph!("vn", |g| {
    g.present("scene")
        // The scene behind everyone. Clears, so it is the one pass that does.
        .pass("backdrop", |p| {
            p.shader(concat!(
                include_str!("vn/common.wgsl"),
                include_str!("vn/backdrop.fs.wgsl"),
            ))
            .color(&["scene"])
            .uniform_of("Stage")
            .clear([0.0, 0.0, 0.0, 1.0])
        })
        // One instance per entity carrying `Actor`. An asset module that
        // publishes a fifth character costs one more spawn and not a line
        // here: the host derives `SeInstance` from the component, so this
        // pass never learns who it drew.
        .pass("actors", |p| {
            p.shader(concat!(
                include_str!("vn/common.wgsl"),
                include_str!("vn/view.wgsl"),
                include_str!("vn/actors.vs.wgsl"),
                include_str!("vn/actors.fs.wgsl"),
            ))
            .color(&["scene"])
            .uniform_of("View")
            .load()
            // The empty mesh name is the built-in unit quad.
            .instanced("Actor", "")
        })
        // The box, over both.
        .pass("box", |p| {
            p.shader(concat!(
                include_str!("vn/common.wgsl"),
                include_str!("vn/view.wgsl"),
                include_str!("vn/box.fs.wgsl"),
            ))
            .color(&["scene"])
            .uniform_of("View")
            .load()
        })
        // The dialogue itself. One instance per `Glyph`, and only the glyphs
        // the control layer decided should exist — this pass has no idea how
        // long the line is or how much of it has been typed out.
        .pass("text", |p| {
            p.shader(concat!(
                include_str!("vn/common.wgsl"),
                include_str!("vn/view.wgsl"),
                include_str!("vn/font.wgsl"),
                include_str!("vn/text.vs.wgsl"),
                include_str!("vn/text.fs.wgsl"),
            ))
            .color(&["scene"])
            .uniform_of("View")
            .load()
            .instanced("Glyph", "")
        })
        // Back to front. Passes nobody ordered keep declaration order anyway,
        // but a layering that only holds by accident is one refactor from a
        // backdrop painted over the cast.
        .edge("backdrop", "actors")
        .edge("actors", "box")
        .edge("box", "text")
});

//! The orthographic look: a backdrop, then every `Sprite` as a rectangle.
//!
//! This module registers nodes and edges and nothing else. It cannot spawn,
//! cannot write a component and knows no time beyond the globals the host
//! injects — so swapping `flat.rs` for a sibling changes how game2 *looks* and
//! is incapable of changing how it plays. That is the whole reason the render
//! side is its own `.so`.
//!
//! ## Why two passes
//!
//! The sky and the ground line are not entities. They have no position to
//! integrate, no hitbox and nothing to recycle, so making them `Sprite`s would
//! put appearance into the roster that `game/game2.rs` reconciles and into the
//! column that `process/scroll.rs` iterates. They are drawn instead by a
//! fullscreen pass behind the sprites, which costs one extra pass and keeps
//! the world to the things the game actually simulates.
//!
//! The `sprites` pass therefore uses `.load()` rather than `.clear()`: it
//! blends over what `sky` left in `scene`, and the `edge` below is what makes
//! "left in `scene`" true this frame rather than last.
//!
//! ## Why no depth buffer
//!
//! `bundle/buffer.rs` declares one target and no depth attachment. In a flat
//! game the sorting is the draw order — the backdrop, then the sprite column —
//! and a depth buffer nothing tests against would be dead weight in the
//! contract. Nothing in game2 overlaps in a way that a z value would resolve
//! better than the order already does.
//!
//! ## The camera
//!
//! `View` is a component read through `uniform_of`, exactly as `Camera` is in
//! game1: the engine has no camera concept, and the projection is
//! `se_ortho` in `flat/view.wgsl` rather than the perspective game1 uses.
//! Both passes name the same uniform because both need to know where the
//! world is on screen.
//!
//! `View.half_w` must agree with `WORLD_HALF_W` in `process/scroll.rs` — that
//! is the constant deciding when a cactus has left the screen — and its ratio
//! to `half_h` should match the aspect the shot is taken at, or the world is
//! drawn stretched. Both are `game/game2.rs`'s to set; this module draws
//! whatever box it is handed.
//!
//! ## The mesh
//!
//! `.instanced("Sprite", "")` — an empty asset name is the engine's built-in
//! unit quad. A flat game draws rectangles, and shipping a four-vertex `.obj`
//! to say so would only add a file that can disagree with the built-in.
//!
//! WGSL lives in `.wgsl` files under `flat/`. That folder is named after this
//! one, so it is private to it: `sky.fs.wgsl` and `sprites.*.wgsl` are this
//! look and no other module can reach them.

se::graph!("flat", |g| {
    g.present("scene")
        // Sky and ground line. Clears `scene`; the fullscreen triangle then
        // covers every pixel, so the clear colour is only ever seen if this
        // pass fails to compile.
        .pass("sky", |p| {
            p.shader(concat!(
                include_str!("flat/view.wgsl"),
                include_str!("flat/sky.fs.wgsl"),
            ))
            .color(&["scene"])
            .uniform_of("View")
            .clear([0.88, 0.91, 0.94, 1.0])
        })
        // The runner, the cacti and anything else carrying a `Sprite`. One
        // instance per entity in the column, in column order.
        .pass("sprites", |p| {
            p.shader(concat!(
                include_str!("flat/view.wgsl"),
                include_str!("flat/sprites.vs.wgsl"),
                include_str!("flat/sprites.fs.wgsl"),
            ))
            .color(&["scene"])
            .uniform_of("View")
            .load()
            .instanced("Sprite", "")
        })
        // Ordering, and here it is load-bearing: `sprites` blends into what
        // `sky` wrote *this* frame, which is a real dependency rather than the
        // previous-frame read that a sampled buffer would give.
        .edge("sky", "sprites")
});

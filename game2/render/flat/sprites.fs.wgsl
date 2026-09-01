// The flat look: a sprite is its tint, with just enough shape to read.
//
// Colour comes from `Sprite.tint` and from nowhere else. The tints themselves
// are not decided here either — `asset/sprites.rs` publishes `palette.txt` and
// `game/game2.rs` writes a role's colour into each entity. So a sibling asset
// module is a new theme, and a sibling render module is a new *look*; this
// file may change how a rectangle is shaded and can neither pick the palette
// nor move anything.

@fragment
fn fs_main(o : VsOut) -> @location(0) vec4<f32> {
    // A gentle top-to-bottom lift. At 100x56 a runner is a handful of pixels
    // and a flat fill reads as a hole in the ground, while a full gradient
    // reads as fog; this is the small amount that says "solid body".
    let up = clamp(o.local.y + 0.5, 0.0, 1.0);
    let c  = o.tint.rgb * mix(0.84, 1.08, up);

    return vec4<f32>(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), o.tint.a);
}

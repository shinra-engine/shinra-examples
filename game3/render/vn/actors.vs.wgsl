// The cast, one instanced unit quad each.
//
// `SeInstance` is generated from `Actor` in bundle/data.rs, so this stage has
// exactly the fields that component declares and could not reach a fifth
// character's extra state even if the asset module invented one. The mesh is
// the built-in unit quad — the pass names `""` — because the engine has no
// portrait concept and this game is not going to teach it one.

struct VnOut {
    @builtin(position) clip : vec4<f32>,
    // -1..1 across the quad, which is where the silhouette is shaped.
    @location(0)       quad : vec2<f32>,
    @location(1)       tint : vec4<f32>,
    // Quad-local units per screen pixel, so the fragment stage can feather an
    // edge without knowing how big this actor ended up.
    @location(2)       fuzz : f32,
    // `Actor.slot` is an index into whatever cast the asset module published.
    // Carried as f32: it is constant per instance, so interpolating it is a
    // no-op, and that avoids a flat-interpolated integer varying.
    @location(3)       slot : f32,
};

@vertex
fn vs_main(v : SeVertex, i : SeInstance) -> VnOut {
    // The unit quad spans -0.5..0.5, so `Actor.size` is a half-extent — the
    // same reading `Glyph.size` gets in bundle/data.rs.
    let world = i.pos + v.pos.xy * 2.0 * i.size;

    let hs = vn_half();
    let proj = se_ortho(hs.x, hs.y, -1.0, 1.0);

    var o : VnOut;
    o.clip = proj * vec4<f32>(world - vn_centre(), 0.0, 1.0);
    o.quad = v.pos.xy * 2.0;
    o.tint = i.tint;
    o.fuzz = vn_texel() / max(i.size.y, 1e-4);
    o.slot = f32(i.slot);
    return o;
}

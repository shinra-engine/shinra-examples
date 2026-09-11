// One unit quad per `Sprite`, placed and sized in world units.
//
// `SeInstance` is generated from the `Sprite` layout in `bundle/data.rs`, so
// `i.pos`, `i.size` and `i.tint` are here because that struct has those
// fields — add one there and it appears here with no binding table to update.
// `SeVertex` is the mesh the pass named; `flat.rs` names the empty mesh, which
// is the engine's built-in unit quad, so `v.pos.xy` spans -0.5..0.5 and the
// game ships no `.obj` at all.
//
// `Sprite.pos` is a *centre* by the definition in `bundle/data.rs`. That is
// the whole of the placement rule: a corner convention here that disagreed
// with the centre-distance test in `process/collide.rs` would draw a runner
// somewhere it cannot be hit.

struct VsOut {
    @builtin(position) clip  : vec4<f32>,
    @location(0)       tint  : vec4<f32>,
    // Position within the quad, -0.5..0.5. Shading only.
    @location(1)       local : vec2<f32>,
    // Where in the atlas this fragment reads.
    @location(2)       uv    : vec2<f32>,
};

@vertex
fn vs_main(v : SeVertex, i : SeInstance) -> VsOut {
    // Scale by the full size, then translate to the centre. No rotation:
    // `Sprite` carries none, because nothing in this game turns.
    let world = v.pos.xy * i.size + i.pos;

    var o : VsOut;
    o.clip  = view_clip(world);
    o.tint  = i.tint;
    o.local = v.pos.xy;
    // The quad spans -0.5..0.5, so shift to 0..1 and map into the cell the
    // instance named. V is flipped: the mesh's +Y is up, an image's is down.
    let t = v.pos.xy + vec2<f32>(0.5, 0.5);
    o.uv = vec2<f32>(
        mix(i.uv.x, i.uv.z, t.x),
        mix(i.uv.w, i.uv.y, t.y),
    );
    return o;
}

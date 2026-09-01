// One instanced unit quad per character.
//
// `SeInstance` is generated from `Glyph` in bundle/data.rs, so this stage sees
// exactly what that component declares: where the character goes, how big it
// is, and — in `uv.x` — which character it is. Placement is the control
// layer's, because only control can read the script; this stage projects what
// it is handed and decides nothing.
//
// Only the revealed prefix of the line is spawned, so the typewriter needs no
// mention here at all. This pass draws what exists.

struct TextOut {
    @builtin(position) clip : vec4<f32>,
    // -1..1 across the quad, which is the frame the glyph cell grid is laid
    // out in.
    @location(0)       quad : vec2<f32>,
    // Character code, carried from `Glyph.uv.x`. Constant per instance, so
    // interpolating it is a no-op and this avoids a flat integer varying —
    // the same reasoning `actors.vs.wgsl` gives for `slot`.
    @location(1)       code : f32,
    // Quad-local units per screen pixel, per axis. The fragment stage needs
    // both: a glyph is 5 cells wide and 7 tall, so how many cells a pixel
    // covers differs between the axes even when the quad is square.
    @location(2)       fuzz : vec2<f32>,
};

@vertex
fn vs_main(v : SeVertex, i : SeInstance) -> TextOut {
    // The unit quad spans -0.5..0.5, so `Glyph.size` is a half-extent — the
    // reading its doc comment in bundle/data.rs gives it, and the one
    // `actors.vs.wgsl` gives `Actor.size`.
    let world = i.pos + v.pos.xy * 2.0 * i.size;

    let hs = vn_half();
    let proj = se_ortho(hs.x, hs.y, -1.0, 1.0);

    var o : TextOut;
    o.clip = proj * vec4<f32>(world - vn_centre(), 0.0, 1.0);
    o.quad = v.pos.xy * 2.0;
    o.code = i.uv.x;
    o.fuzz = vn_texel() / max(i.size, vec2<f32>(1e-4, 1e-4));
    return o;
}

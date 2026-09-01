// The dialogue box: a panel across the bottom, with a name plate on it.
//
// Fullscreen rather than instanced, because there is no box component to
// instance and there is not going to be one — the box is furniture, and
// bundle/data.rs is a contract the whole bundle already agreed on. So it is
// laid out against the *view rect* instead: the panel keeps its place when
// `View` drifts or the window changes shape, which is what furniture does.
//
// The glyphs that go inside it are a separate pass in a separate ticket. What
// this pass owes them is the geometry below, in view-relative world units.

// Fractions of the view rect. Every number a game needs lives in the game.
const BOX_BOTTOM : f32 = 0.62;   // panel centre, in half-heights below centre
const BOX_HALF_W : f32 = 0.90;   // panel half-width,  in half-widths
const BOX_HALF_H : f32 = 0.26;   // panel half-height, in half-heights
const PLATE_HALF_W : f32 = 0.21;
const PLATE_HALF_H : f32 = 0.052;

@fragment
fn fs_main(o : SeVsOut) -> @location(0) vec4<f32> {
    let hs = vn_half();
    // `SeVsOut.uv` is 0..1, y down. The view rect is centred on `View`, so
    // dropping the centre leaves view-relative world units — which is the
    // frame the constants above are written in.
    let p = vec2<f32>(o.uv.x * 2.0 - 1.0, 1.0 - o.uv.y * 2.0) * hs;
    let w = max(vn_texel(), 1e-5);

    let c = vec2<f32>(0.0, -hs.y * BOX_BOTTOM);
    let half = vec2<f32>(hs.x * BOX_HALF_W, hs.y * BOX_HALF_H);
    let r = min(half.x, half.y) * 0.22;
    let d = sd_round_box(p - c, half, r);

    // The name plate sits on the panel's top-left corner, where the speaker's
    // name will go once the glyph pass lands.
    let pc = vec2<f32>(c.x - half.x + hs.x * PLATE_HALF_W + hs.x * 0.03,
                       c.y + half.y);
    let ph = vec2<f32>(hs.x * PLATE_HALF_W, hs.y * PLATE_HALF_H);
    let pd = sd_round_box(p - pc, ph, min(ph.x, ph.y) * 0.35);

    // A vertical wash, so the panel is not a flat rectangle at a glance.
    let g = clamp((c.y + half.y - p.y) / max(2.0 * half.y, 1e-4), 0.0, 1.0);

    var col = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    // Body, then its border, then the plate, then the plate's border. Painted
    // in that order because each sits on top of the last; `over` composites
    // them into the one straight-alpha colour the target blends.
    col = over(col, vec4<f32>(mix(PLATE * 1.9, PLATE * 0.6, g), band(d, w) * 0.88));
    col = over(col, vec4<f32>(EDGE, band(abs(d + r * 0.10), w * 1.5) * 0.80));
    col = over(col, vec4<f32>(PLATE * 2.6, band(pd, w) * 0.95));
    col = over(col, vec4<f32>(EMBER, band(abs(pd + ph.y * 0.16), w * 1.5) * 0.85));
    return col;
}

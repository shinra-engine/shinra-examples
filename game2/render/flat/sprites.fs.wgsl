// Sprites, sampled from the atlas an asset module published.
//
// The shader names the texture `atlas` because the pass declared
// `.textures(&["atlas.png"])` — the engine strips the extension and binds it.
// Nothing here knows which file that was, so a different `asset/*.so`
// publishing `atlas.png` reskins the game without this file changing.

@fragment
fn fs_main(o : VsOut) -> @location(0) vec4<f32> {
    let texel = textureSample(atlas, se_sampler, o.uv);

    // Fully transparent texels are the sheet's empty space, not black paint.
    if (texel.a < 0.02) {
        discard;
    }

    // Tint multiplies, so a white cell shows the artwork as drawn.
    var c = texel.rgb * o.tint.rgb;

    // A touch of vertical shading so a flat sprite still reads as solid
    // against the flat sky.
    c *= 0.88 + 0.12 * (0.5 - o.local.y);

    return vec4<f32>(c, texel.a * o.tint.a);
}

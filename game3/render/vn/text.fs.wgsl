// Turning a character code into ink.
//
// The quad is the glyph's cell grid: 5 across, 7 down, origin top-left, which
// is the frame `font.wgsl` writes its bit patterns in. Testing one bit per
// fragment is what a font atlas would have done by sampling, except the table
// is constants because the engine has no path from an asset to a texture.
//
// Antialiasing is not optional here. A dialogue glyph is a few pixels tall in
// a `shot`, and a single bit test at that size is a coin flip per pixel that
// changes as the view drifts. Four taps carry it while a pixel is smaller
// than a cell; below that the glyph fades to its own average density, so an
// unreadably small line degrades into a weighted smudge instead of noise.

// Text ink, and the ghost of it that keeps a glyph off the panel's own
// colour. Both are this pass's, not the palette's: `common.wgsl` describes
// the scene, and how readable dialogue has to be is a different question.
const TEXT_INK   : vec3<f32> = vec3<f32>(0.94, 0.95, 0.99);
const TEXT_SHADE : vec3<f32> = vec3<f32>(0.02, 0.02, 0.05);

// Pixels per cell at which four taps stop being enough. Between these the
// crisp shape cross-fades into the glyph's density.
const SHARP_AT : f32 = 1.20;
const SMUDGE_AT : f32 = 0.45;

@fragment
fn fs_main(o : TextOut) -> @location(0) vec4<f32> {
    let idx = font_index(u32(max(o.code, 0.0)));
    let grid = vec2<f32>(GLYPH_COLS, GLYPH_ROWS);

    // Quad -1..1 to cell coordinates, y down, because row 0 is the top row.
    let cell = vec2<f32>(o.quad.x * 0.5 + 0.5, 0.5 - o.quad.y * 0.5) * grid;

    // Cells one pixel covers. `fuzz` is quad units per pixel and the quad
    // spans two units across `grid` cells, hence the half.
    let px = o.fuzz * grid * 0.5;

    // Four taps on a rotated grid — offset on both axes, so a horizontal bar
    // and a vertical stem are sampled alike.
    let d = px * 0.25;
    var ink = font_ink(idx, cell + vec2<f32>(-d.x, -d.y * 0.5));
    ink += font_ink(idx, cell + vec2<f32>(d.x * 0.5, -d.y));
    ink += font_ink(idx, cell + vec2<f32>(d.x, d.y * 0.5));
    ink += font_ink(idx, cell + vec2<f32>(-d.x * 0.5, d.y));
    ink *= 0.25;

    // Too small to resolve: fade towards the glyph's average ink. The blend
    // runs on the taller axis, which is the one that runs out of pixels first.
    let resolved = smoothstep(SMUDGE_AT, SHARP_AT, 1.0 / max(px.y, 1e-4));
    let cover = mix(font_density(idx), ink, resolved);

    if (cover <= 0.001) {
        discard;
    }

    // A dark halo under the ink, weighted by how much of the neighbourhood is
    // lit. It costs nothing where the glyph is empty and keeps a light letter
    // legible if a later look brightens the panel underneath it.
    let halo = clamp(cover * 1.6, 0.0, 1.0);
    let col = mix(TEXT_SHADE, TEXT_INK, cover);
    return vec4<f32>(col, halo);
}

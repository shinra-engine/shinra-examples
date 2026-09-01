// The font. A bit table, because there is nowhere else it could live.
//
// The engine creates textures only from `bundle/buffer.rs`, and `asset/*.so`
// bytes reach the renderer only as `.obj` meshes — so there is no path from a
// `.png` to something a shader can sample, and an atlas is not expressible.
// A glyph is therefore *shaped* rather than sampled, and the shape has to be
// constants in the shader.
//
// 5 wide by 7 tall, which is 35 bits, and 35 bits do not fit in a `u32`. So
// each glyph is a pair: `FONT_LO` carries rows 0..3 and `FONT_HI` rows 4..6,
// both with bit `row_in_group * 5 + col` set where there is ink, column 0 at
// the left and row 0 at the top. The art in the comments is that same bit
// pattern written out, which is the only way this table stays reviewable.
//
// The set is the one `story.rpy` writes its dialogue in: space, 0-9, A-Z and
// `.,!?'-:`. Lowercase folds to uppercase — a 5x7 cell has no room for
// descenders — and anything else, including the second byte of a multi-byte
// character, draws as blank rather than as a box. A story that wants a wider
// alphabet grows this table; it does not grow the engine.

// Cells per glyph. Also the coordinate system `text.fs.wgsl` tests bits in.
const GLYPH_COLS : f32 = 5.0;
const GLYPH_ROWS : f32 = 7.0;

// Cells in one glyph. `font_density` divides by it.
const FONT_CELLS : f32 = 35.0;

// Rows 0..3.
const FONT_LO : array<u32, 44> = array<u32, 44>(
    0x00000u,  // space ..... ..... ..... .....
    0xae62eu,  // 0     .###. #...# #..## #.#.#
    0x210c4u,  // 1     ..#.. .##.. ..#.. ..#..
    0x4422eu,  // 2     .###. #...# ....# ...#.
    0x8311fu,  // 3     ##### ...#. ..##. ....#
    0x4a988u,  // 4     ...#. ..##. .#.#. #..#.
    0x83c3fu,  // 5     ##### #.... ####. ....#
    0x7844cu,  // 6     ..##. .#... #.... ####.
    0x2221fu,  // 7     ##### ....# ...#. ..#..
    0x7462eu,  // 8     .###. #...# #...# .###.
    0xf462eu,  // 9     .###. #...# #...# .####
    0xfc62eu,  // A     .###. #...# #...# #####
    0x7c62fu,  // B     ####. #...# #...# ####.
    0x0862eu,  // C     .###. #...# #.... #....
    0x8c62fu,  // D     ####. #...# #...# #...#
    0x7843fu,  // E     ##### #.... #.... ####.
    0x7843fu,  // F     ##### #.... #.... ####.
    0xe862eu,  // G     .###. #...# #.... #.###
    0xfc631u,  // H     #...# #...# #...# #####
    0x2108eu,  // I     .###. ..#.. ..#.. ..#..
    0x4211cu,  // J     ..### ...#. ...#. ...#.
    0x19531u,  // K     #...# #..#. #.#.. ##...
    0x08421u,  // L     #.... #.... #.... #....
    0x8d771u,  // M     #...# ##.## #.#.# #...#
    0xcd671u,  // N     #...# ##..# #.#.# #..##
    0x8c62eu,  // O     .###. #...# #...# #...#
    0x7c62fu,  // P     ####. #...# #...# ####.
    0x8c62eu,  // Q     .###. #...# #...# #...#
    0x7c62fu,  // R     ####. #...# #...# ####.
    0x7043eu,  // S     .#### #.... #.... .###.
    0x2109fu,  // T     ##### ..#.. ..#.. ..#..
    0x8c631u,  // U     #...# #...# #...# #...#
    0x8c631u,  // V     #...# #...# #...# #...#
    0x8c631u,  // W     #...# #...# #...# #...#
    0x22a31u,  // X     #...# #...# .#.#. ..#..
    0x22a31u,  // Y     #...# #...# .#.#. ..#..
    0x2221fu,  // Z     ##### ....# ...#. ..#..
    0x00000u,  // .     ..... ..... ..... .....
    0x00000u,  // ,     ..... ..... ..... .....
    0x21084u,  // !     ..#.. ..#.. ..#.. ..#..
    0x4422eu,  // ?     .###. #...# ....# ...#.
    0x00884u,  // quote ..#.. ..#.. .#... .....
    0xf8000u,  // -     ..... ..... ..... #####
    0x01080u,  // :     ..... ..#.. ..#.. .....
);

// Rows 4..6.
const FONT_HI : array<u32, 44> = array<u32, 44>(
    0x0000u,   // space ..... ..... .....
    0x3a33u,   // 0     ##..# #...# .###.
    0x3884u,   // 1     ..#.. ..#.. .###.
    0x7c44u,   // 2     ..#.. .#... #####
    0x3a30u,   // 3     ....# #...# .###.
    0x211fu,   // 4     ##### ...#. ...#.
    0x3a30u,   // 5     ....# #...# .###.
    0x3a31u,   // 6     #...# #...# .###.
    0x0842u,   // 7     .#... .#... .#...
    0x3a31u,   // 8     #...# #...# .###.
    0x1910u,   // 9     ....# ...#. .##..
    0x4631u,   // A     #...# #...# #...#
    0x3e31u,   // B     #...# #...# ####.
    0x3a21u,   // C     #.... #...# .###.
    0x3e31u,   // D     #...# #...# ####.
    0x7c21u,   // E     #.... #.... #####
    0x0421u,   // F     #.... #.... #....
    0x3a31u,   // G     #...# #...# .###.
    0x4631u,   // H     #...# #...# #...#
    0x3884u,   // I     ..#.. ..#.. .###.
    0x1928u,   // J     ...#. #..#. .##..
    0x4525u,   // K     #.#.. #..#. #...#
    0x7c21u,   // L     #.... #.... #####
    0x4631u,   // M     #...# #...# #...#
    0x4631u,   // N     #...# #...# #...#
    0x3a31u,   // O     #...# #...# .###.
    0x0421u,   // P     #.... #.... #....
    0x5935u,   // Q     #.#.# #..#. .##.#
    0x4525u,   // R     #.#.. #..#. #...#
    0x3e10u,   // S     ....# ....# ####.
    0x1084u,   // T     ..#.. ..#.. ..#..
    0x3a31u,   // U     #...# #...# .###.
    0x1151u,   // V     #...# .#.#. ..#..
    0x4775u,   // W     #.#.# ##.## #...#
    0x462au,   // X     .#.#. #...# #...#
    0x1084u,   // Y     ..#.. ..#.. ..#..
    0x7c22u,   // Z     .#... #.... #####
    0x18c0u,   // .     ..... .##.. .##..
    0x08c6u,   // ,     .##.. .##.. .#...
    0x1004u,   // !     ..#.. ..... ..#..
    0x1004u,   // ?     ..#.. ..... ..#..
    0x0000u,   // quote ..... ..... .....
    0x0000u,   // -     ..... ..... .....
    0x0084u,   // :     ..#.. ..#.. .....
);

// Where a character code lands in the table. Codes the table has no glyph for
// answer 0, which is the blank — a dialogue line is data from an asset module
// and this pass is not entitled to reject it.
fn font_index(code : u32) -> u32 {
    var c = code;
    // Lowercase folds up. A 5x7 cell cannot hold a descender, so `g` and `q`
    // would be lies anyway.
    if (c >= 97u && c <= 122u) { c = c - 32u; }

    if (c >= 48u && c <= 57u) { return c - 48u + 1u; }   // 0-9
    if (c >= 65u && c <= 90u) { return c - 65u + 11u; }  // A-Z

    switch (c) {
        case 46u: { return 37u; }  // .
        case 44u: { return 38u; }  // ,
        case 33u: { return 39u; }  // !
        case 63u: { return 40u; }  // ?
        case 39u: { return 41u; }  // '
        case 45u: { return 42u; }  // -
        case 58u: { return 43u; }  // :
        default:  { return 0u; }
    }
}

// Ink at one cell, 0 or 1. Out-of-range cells are blank, so a caller may
// sample past the glyph's edge without guarding first.
fn font_ink(idx : u32, cell : vec2<f32>) -> f32 {
    if (cell.x < 0.0 || cell.y < 0.0 || cell.x >= GLYPH_COLS || cell.y >= GLYPH_ROWS) {
        return 0.0;
    }
    let col = u32(cell.x);
    let row = u32(cell.y);
    if (row < 4u) {
        return f32((FONT_LO[idx] >> (row * 5u + col)) & 1u);
    }
    return f32((FONT_HI[idx] >> ((row - 4u) * 5u + col)) & 1u);
}

// How much of the cell grid this glyph fills, 0..1.
//
// Below about one pixel per cell no amount of sampling recovers the letter
// shape, and four taps of a 5x7 grid at that size is noise that flickers as
// the view moves. Fading to the glyph's average density instead gives a
// stable block whose weight still tracks the character — a smudge that reads
// as text rather than as a broken shader.
fn font_density(idx : u32) -> f32 {
    let n = countOneBits(FONT_LO[idx]) + countOneBits(FONT_HI[idx]);
    return f32(n) / FONT_CELLS;
}

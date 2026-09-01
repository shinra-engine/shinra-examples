// Palette and shapes shared by every pass in `vn.rs`.
//
// This folder is named after a sibling `.rs`, so it is private to that module:
// these colours are what makes the `vn` look the `vn` look, and a second
// render module for game3 would be free to disagree with all of them.
//
// Nothing here touches `u`. The three passes name three different components
// as their uniform, so a helper that read `u` could only be included by some
// of them — the view helpers live in `view.wgsl` for exactly that reason.

const INK   : vec3<f32> = vec3<f32>(0.03, 0.04, 0.07);
const DUSK  : vec3<f32> = vec3<f32>(0.11, 0.14, 0.28);
const HAZE  : vec3<f32> = vec3<f32>(0.45, 0.31, 0.48);
const EMBER : vec3<f32> = vec3<f32>(0.98, 0.72, 0.46);
const PLATE : vec3<f32> = vec3<f32>(0.07, 0.09, 0.17);
const EDGE  : vec3<f32> = vec3<f32>(0.55, 0.78, 1.00);

// Signed distance to a rounded box centred on the origin. `hs` is the
// half-extent including the corner radius, so `r` must not exceed either axis.
fn sd_round_box(p : vec2<f32>, hs : vec2<f32>, r : f32) -> f32 {
    let q = abs(p) - hs + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

// Coverage of a distance field, feathered over `w` units either side of the
// edge. `w` is in the same units as `d`, so a caller working in world units
// scales it by the view — a fixed pixel width is not expressible here and
// would be wrong the moment the view rect changed.
fn band(d : f32, w : f32) -> f32 {
    return 1.0 - smoothstep(-w, w, d);
}

// Straight-alpha `src` over straight-alpha `dst`.
//
// The colour target blends with `ALPHA_BLENDING`, so a pass hands back one
// straight-alpha colour. A pass that draws several shapes therefore has to
// composite them itself before returning, which is what this is for.
fn over(dst : vec4<f32>, src : vec4<f32>) -> vec4<f32> {
    let a = src.a + dst.a * (1.0 - src.a);
    let rgb = (src.rgb * src.a + dst.rgb * dst.a * (1.0 - src.a)) / max(a, 1e-4);
    return vec4<f32>(rgb, a);
}

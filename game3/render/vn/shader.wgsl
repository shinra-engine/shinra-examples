// game3's flat look. `SpriteInstance` and `View` come from contract.wgsl,
// which the host prepends: the instance input *is* the component record, at
// the record's own std430 offsets.
//
// No atlas yet. `uv` rides in the record and reaches here untouched, so the
// day an asset module publishes a sheet the sampling is the only line that
// changes.

@group(0) @binding(0) var<uniform> view: View;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) tint: vec4<f32>,
    @location(1) quad: vec2<f32>,
    @location(2) slot: f32,
};

const INK: vec3<f32> = vec3<f32>(0.03, 0.04, 0.07);
const DUSK: vec3<f32> = vec3<f32>(0.11, 0.14, 0.28);
const HAZE: vec3<f32> = vec3<f32>(0.45, 0.31, 0.48);
const EMBER: vec3<f32> = vec3<f32>(0.98, 0.72, 0.46);
const EDGE: vec3<f32> = vec3<f32>(0.55, 0.78, 1.00);

// Signed distance to a rounded box centred on the origin. `hs` includes the
// corner radius.
fn sd_round_box(p: vec2<f32>, hs: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - hs + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

// Coverage of a distance field, feathered over `w` either side of the edge.
fn band(d: f32, w: f32) -> f32 {
    return 1.0 - smoothstep(-w, w, d);
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, inst: SpriteInstance) -> VsOut {
    // Two triangles, as a quad around the sprite's centre.
    var corner = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.5), vec2<f32>( 0.5, -0.5), vec2<f32>( 0.5,  0.5),
        vec2<f32>(-0.5, -0.5), vec2<f32>( 0.5,  0.5), vec2<f32>(-0.5,  0.5),
    );
    let c = corner[vi];
    let world = inst.pos + c * inst.size;

    // Orthographic: the visible box is half-extents around a centre, so the
    // projection is two divides and nothing else.
    let ndc = vec2<f32>(
        (world.x - view.cx) / max(view.half_w, 0.0001),
        (world.y - view.cy) / max(view.half_h, 0.0001),
    );

    var o: VsOut;
    o.clip = vec4<f32>(ndc, 0.0, 1.0);
    o.tint = inst.tint;
    o.quad = c * 2.0;
    o.slot = inst.uv.x;
    return o;
}

@fragment
fn fs_main(i: VsOut) -> @location(0) vec4<f32> {
    let p = i.quad;

    // A head over a body, unioned.
    let body = sd_round_box(p - vec2<f32>(0.0, -0.35), vec2<f32>(0.70, 0.64), 0.42);
    let head = sd_round_box(p - vec2<f32>(0.0, 0.62), vec2<f32>(0.34, 0.34), 0.32);
    let d = min(body, head);

    // No blending in the pipeline: outside the shape there is nothing to draw.
    let coverage = band(d, 0.02);
    if (coverage < 0.5) {
        discard;
    }

    // Hue by slot, so the cast is distinguishable without being told a name.
    let hue = fract(i.slot * 0.37 + 0.11);
    var col = mix(HAZE, EMBER, hue) * 0.75 + DUSK * 0.5;
    col *= 0.62 + 0.38 * clamp(0.55 - p.x * 0.45 - p.y * 0.18, 0.0, 1.0);

    // A rim just inside the silhouette keeps a dimmed actor readable.
    col += EDGE * band(abs(d + 0.05), 0.045) * 0.55;

    // The stage's tint carries who is lit and the fade; the shader never reads `line`.
    return vec4<f32>(col * i.tint.rgb, 1.0);
}

// game2's flat look. `SpriteInstance` and `View` come from contract.wgsl,
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
    @location(1) uv: vec2<f32>,
};

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
    o.uv = vec2<f32>(
        mix(inst.uv.x, inst.uv.z, c.x + 0.5),
        mix(inst.uv.y, inst.uv.w, 0.5 - c.y),
    );
    return o;
}

@fragment
fn fs_main(i: VsOut) -> @location(0) vec4<f32> {
    // A soft edge, so a rectangle reads as a shape rather than a block.
    let d = abs(i.uv - vec2<f32>(0.5, 0.5)) * 2.0;
    let edge = 1.0 - smoothstep(0.86, 1.0, max(d.x, d.y));
    return vec4<f32>(i.tint.rgb * (0.75 + 0.25 * edge), i.tint.a);
}

// game1's solid look. `TransformInstance` and `Camera` are not declared here:
// the host prepends contract.wgsl, so this shader's instance input *is* the
// component record, at the record's own std430 offsets. Add a field to
// contract.wit and it appears here.

@group(0) @binding(0) var<uniform> cam: Camera;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) tint: vec3<f32>,
};

/// View matrix: right-handed, looking down -z, as WebGPU expects.
fn view_of(eye: vec3<f32>, at: vec3<f32>) -> mat4x4<f32> {
    let f = normalize(at - eye);
    let s = normalize(cross(f, vec3<f32>(0.0, 1.0, 0.0)));
    let u = cross(s, f);
    // WGSL's mat4x4 takes *columns*.
    return mat4x4<f32>(
        vec4<f32>(s.x, u.x, -f.x, 0.0),
        vec4<f32>(s.y, u.y, -f.y, 0.0),
        vec4<f32>(s.z, u.z, -f.z, 0.0),
        vec4<f32>(-dot(s, eye), -dot(u, eye), dot(f, eye), 1.0),
    );
}

/// Perspective onto WebGPU's 0..1 depth range.
///
/// The aspect ratio is a constant here because the camera is a control var and
/// the control module does not know the framebuffer. Publishing the viewport
/// as a control var would fix that properly; until then a wide-ish default is
/// wrong by less than the eye notices.
fn proj_of(fov: f32) -> mat4x4<f32> {
    let aspect = 1.6;
    let g = 1.0 / tan(fov * 0.5);
    let n = 0.1;
    let f = 100.0;
    return mat4x4<f32>(
        vec4<f32>(g / aspect, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, g, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, f / (n - f), -1.0),
        vec4<f32>(0.0, 0.0, n * f / (n - f), 0.0),
    );
}

fn rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let t = 2.0 * cross(q.xyz, v);
    return v + q.w * t + cross(q.xyz, t);
}

/// Three models, chosen per row by `transform.kind`.
///
/// They are built here rather than loaded because the point being made is
/// that the *choice* crosses the boundary as data: the control layer writes a
/// number, a stage copies it onto every row, and it arrives as instance
/// input. Where the vertices come from is this shader's business — an asset
/// module publishing them would change nothing above this line.
fn model_vertex(kind: u32, vi: u32) -> vec3<f32> {
    var tetra = array<vec3<f32>, 12>(
        vec3<f32>( 0.0,  0.8,  0.0), vec3<f32>(-0.7, -0.4,  0.4), vec3<f32>( 0.7, -0.4,  0.4),
        vec3<f32>( 0.0,  0.8,  0.0), vec3<f32>( 0.7, -0.4,  0.4), vec3<f32>( 0.0, -0.4, -0.8),
        vec3<f32>( 0.0,  0.8,  0.0), vec3<f32>( 0.0, -0.4, -0.8), vec3<f32>(-0.7, -0.4,  0.4),
        vec3<f32>(-0.7, -0.4,  0.4), vec3<f32>( 0.0, -0.4, -0.8), vec3<f32>( 0.7, -0.4,  0.4),
    );
    var spike = array<vec3<f32>, 12>(
        vec3<f32>( 0.0,  1.3,  0.0), vec3<f32>(-0.35, -0.6,  0.35), vec3<f32>( 0.35, -0.6,  0.35),
        vec3<f32>( 0.0,  1.3,  0.0), vec3<f32>( 0.35, -0.6,  0.35), vec3<f32>( 0.35, -0.6, -0.35),
        vec3<f32>( 0.0,  1.3,  0.0), vec3<f32>( 0.35, -0.6, -0.35), vec3<f32>(-0.35, -0.6, -0.35),
        vec3<f32>( 0.0,  1.3,  0.0), vec3<f32>(-0.35, -0.6, -0.35), vec3<f32>(-0.35, -0.6,  0.35),
    );
    var plate = array<vec3<f32>, 12>(
        vec3<f32>(-0.9,  0.05, -0.9), vec3<f32>( 0.9,  0.05, -0.9), vec3<f32>( 0.9,  0.05,  0.9),
        vec3<f32>(-0.9,  0.05, -0.9), vec3<f32>( 0.9,  0.05,  0.9), vec3<f32>(-0.9,  0.05,  0.9),
        vec3<f32>(-0.9, -0.05,  0.9), vec3<f32>( 0.9, -0.05,  0.9), vec3<f32>( 0.9, -0.05, -0.9),
        vec3<f32>(-0.9, -0.05,  0.9), vec3<f32>( 0.9, -0.05, -0.9), vec3<f32>(-0.9, -0.05, -0.9),
    );
    if (kind == 1u) { return spike[vi]; }
    if (kind == 2u) { return plate[vi]; }
    return tetra[vi];
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, inst: TransformInstance) -> VsOut {
    let local = model_vertex(inst.kind, vi) * inst.scale;
    let world = rotate(inst.rot, local) + inst.pos;

    let tri = vi / 3u;
    var o: VsOut;
    o.clip = proj_of(cam.fov) * view_of(cam.eye, cam.focus) * vec4<f32>(world, 1.0);
    o.normal = normalize(rotate(inst.rot, model_vertex(inst.kind, vi)));
    o.tint = vec3<f32>(
        0.45 + 0.35 * f32(tri % 2u),
        0.55 + 0.20 * f32((tri + 1u) % 3u) / 2.0,
        0.85 - 0.25 * f32(tri % 3u) / 2.0,
    );
    return o;
}

@fragment
fn fs_main(i: VsOut) -> @location(0) vec4<f32> {
    let key = normalize(vec3<f32>(0.4, 0.8, 0.3));
    let lit = 0.55 + 0.45 * abs(dot(normalize(i.normal), key));
    return vec4<f32>(vec3<f32>(1.0, 0.85, 0.95) * lit * 0.9, 1.0);
}

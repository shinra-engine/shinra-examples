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

/// Meshes arrive as vertex buffer 1 (`MeshVertex`); the engine does not
/// normalise them, so each is scaled to a common size by `inst.kind`, which
/// `process/model.rs` copies from `look.asset`: bunny x6.0 (spans ~0.16),
/// quad x1.0, teapot x0.25 (spans ~+-3).
fn mesh_scale(kind: u32) -> f32 {
    if (kind == 0u) { return 6.0; }
    if (kind == 1u) { return 1.0; }
    return 0.25;
}

@vertex
fn vs_main(inst: TransformInstance, v: MeshVertex) -> VsOut {
    let local = v.pos * (inst.scale * mesh_scale(inst.kind));
    let world = rotate(inst.rot, local) + inst.pos;

    var o: VsOut;
    o.clip = proj_of(cam.fov) * view_of(cam.eye, cam.focus) * vec4<f32>(world, 1.0);
    o.normal = normalize(rotate(inst.rot, v.normal));
    o.tint = vec3<f32>(
        0.45 + 0.35 * f32(inst.kind % 2u),
        0.55 + 0.20 * f32((inst.kind + 1u) % 3u) / 2.0,
        0.85 - 0.25 * f32(inst.kind % 3u) / 2.0,
    );
    return o;
}

@fragment
fn fs_main(i: VsOut) -> @location(0) vec4<f32> {
    let key = normalize(vec3<f32>(0.4, 0.8, 0.3));
    let lit = 0.55 + 0.45 * abs(dot(normalize(i.normal), key));
    return vec4<f32>(vec3<f32>(1.0, 0.85, 0.95) * lit * 0.9, 1.0);
}

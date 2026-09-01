// Fragment stage for the bodies in the `ghost` look.
//
// Flat and bright, so the accumulator has something with edges to smear.

@fragment
fn fs_main(o : VsOut) -> @location(0) vec4<f32> {
    let n = normalize(o.normal);
    let k = key_light(n) * 0.6 + 0.4;
    let c = vec3<f32>(0.35, 0.85, 1.0) * k
          + vec3<f32>(1.0, 0.45, 0.9) * rim(n, o.world);
    return vec4<f32>(c, 1.0);
}

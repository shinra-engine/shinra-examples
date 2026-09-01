// Lighting helpers shared across the render category.
//
// Deliberately cheap: this engine renders on a software rasteriser often
// enough that a fragment shader is not the place to be clever.

fn key_light(n : vec3<f32>) -> f32 {
    let l = normalize(vec3<f32>(0.4, 0.8, 0.45));
    return clamp(dot(n, l), 0.0, 1.0);
}

fn rim(n : vec3<f32>, world : vec3<f32>) -> f32 {
    let v = normalize(u.eye.xyz - world);
    return pow(1.0 - clamp(dot(n, v), 0.0, 1.0), 2.5);
}

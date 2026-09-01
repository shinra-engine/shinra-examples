// This frame's bodies over a fading image of themselves.
//
// Both `scene` and `trail` arrive as previous-frame content — a sampled buffer
// always does — which is why this pass may read `trail` while writing it. It
// is a one-frame feedback loop, not a cycle, and the decay below is how fast
// the past fades.

@fragment
fn fs_main(o : SeVsOut) -> @location(0) vec4<f32> {
    let now  = textureSample(scene, se_sampler, o.uv);
    let past = textureSample(trail, se_sampler, o.uv);

    // Brightest-wins keeps a clean streak; averaging would smear to grey.
    let keep = past.rgb * 0.90 - vec3<f32>(0.004);
    return vec4<f32>(max(now.rgb, max(keep, vec3<f32>(0.0))), 1.0);
}

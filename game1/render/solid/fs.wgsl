// Fragment stage for the `solid` look. Private to render/solid.rs.
//
// A warm key over a cool ambient, plus a rim so a silhouette still reads when
// the key is behind the body. This palette is what makes `solid` look like
// `solid`; `ghost.rs` deliberately cannot reach it.

const COOL : vec3<f32> = vec3<f32>(0.09, 0.11, 0.20);
const WARM : vec3<f32> = vec3<f32>(1.00, 0.86, 0.62);
const RIMC : vec3<f32> = vec3<f32>(0.35, 0.70, 1.00);

@fragment
fn fs_main(o : VsOut) -> @location(0) vec4<f32> {
    let n = normalize(o.normal);
    let k = key_light(n);

    // Wrap the terminator a little; a hard one looks wrong at this scale.
    let wrapped = k * 0.75 + 0.25 * clamp(n.y * 0.5 + 0.5, 0.0, 1.0);

    var c = mix(COOL, WARM, wrapped);
    c += RIMC * rim(n, o.world) * 0.45;

    // Fog toward the clear colour so distant bodies sit behind near ones even
    // where the depth buffer alone would not read.
    let fog = clamp((length(o.world - u.eye.xyz) - 3.0) / 14.0, 0.0, 1.0);
    c = mix(c, COOL * 0.6, fog * 0.55);

    return vec4<f32>(c, 1.0);
}

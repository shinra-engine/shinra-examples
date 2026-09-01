// The backdrop: sky above the ground line, sand below it.
//
// A fullscreen pass, so the engine supplies the vertex stage (`se_vs_fullscreen`)
// and this file is only a fragment. It draws no entity and reads no buffer —
// it needs the world only to know where the horizon is, which is `View`
// arriving as the uniform and `GROUND_Y` below.
//
// Nothing here is a rule. The ground line is *drawn* at the height the runner
// lands at; what makes the runner land is `process/gravity.rs`, and swapping
// this module for a night-time one must not change where it stops falling.

/// World y of the ground surface.
///
/// The one number in this module that has to agree with something outside it:
/// `GROUND_Y` in `process/gravity.rs` is where a body actually rests, and this
/// is where the line saying so is painted. A render module may not read a
/// stage's constant and a stage may not read this one, so the two are changed
/// together or the runner floats.
const GROUND_Y : f32 = 0.0;

const SKY_TOP  : vec3<f32> = vec3<f32>(0.62, 0.74, 0.88);
const SKY_LOW  : vec3<f32> = vec3<f32>(0.88, 0.91, 0.94);
const SAND     : vec3<f32> = vec3<f32>(0.78, 0.74, 0.66);
const SAND_LOW : vec3<f32> = vec3<f32>(0.66, 0.62, 0.55);
const LINE     : vec3<f32> = vec3<f32>(0.33, 0.33, 0.33);

@fragment
fn fs_main(o : SeVsOut) -> @location(0) vec4<f32> {
    let world = view_world(o.uv);
    let half  = view_half();

    // Sky: pale at the horizon, deeper overhead. Measured against the visible
    // box rather than a fixed altitude, so the gradient survives a camera that
    // is widened or moved.
    let up  = clamp((world.y - GROUND_Y) / max(half.y * 2.0, 0.001), 0.0, 1.0);
    var c   = mix(SKY_LOW, SKY_TOP, up);

    // Ground: darkening with depth, which is downwards in a flat game.
    let down = clamp((GROUND_Y - world.y) / max(half.y, 0.001), 0.0, 1.0);
    let dirt = mix(SAND, SAND_LOW, down);
    c = select(c, dirt, world.y < GROUND_Y);

    // The line itself, a hairline at the surface. `view_pixel` keeps it one
    // pixel at any resolution — at 100x56 a line measured in world units
    // would either vanish or become a band.
    let px = view_pixel();
    c = select(c, LINE, abs(world.y - GROUND_Y) < px);

    return vec4<f32>(c, 1.0);
}

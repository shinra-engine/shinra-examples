// Turning the `View` component into an orthographic projection.
//
// Shared by both passes of this module and by nothing else: `render/flat/` is
// named after `flat.rs`, so it is private to it. The engine has no camera —
// `View` is a component like any other, handed to a pass as `uniform_of`, and
// this file is the whole of what "camera" means to game2.
//
// Uniform fields arrive repacked into whole `vec4` slots, so a scalar field of
// `View` is read as `.x`. That is why every accessor below exists: writing
// `u.half_w.x` at each use site reads like a mistake, and `view_half().x`
// does not.

/// Centre of the visible box, world units.
fn view_centre() -> vec2<f32> {
    return vec2<f32>(u.cx.x, u.cy.x);
}

/// Half-width and half-height of the visible box, world units.
fn view_half() -> vec2<f32> {
    return vec2<f32>(u.half_w.x, u.half_h.x);
}

/// World -> clip. `se_ortho` boxes the world around the origin, so the camera
/// centre is subtracted first; z is flat because game2 has no depth buffer and
/// draw order is the only sorting there is.
fn view_clip(world : vec2<f32>) -> vec4<f32> {
    let half = view_half();
    let proj = se_ortho(half.x, half.y, -1.0, 1.0);
    return proj * vec4<f32>(world - view_centre(), 0.0, 1.0);
}

/// Fullscreen uv -> world. `SeVsOut.uv` has y running down from the top, so
/// the vertical flip is here rather than repeated in a fragment stage.
fn view_world(uv : vec2<f32>) -> vec2<f32> {
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    return view_centre() + ndc * view_half();
}

/// Height of one screen pixel, in world units. A hairline drawn in world
/// space stays one pixel wide at any `--size` the shot is taken at.
fn view_pixel() -> f32 {
    return 2.0 * view_half().y / max(se.resolution.y, 1.0);
}

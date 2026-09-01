// The orthographic view, for the passes that name `View` as their uniform.
//
// A visual novel is a flat stage, so there is no camera to look through —
// just a rectangle of world that the screen shows. `View` in bundle/data.rs
// is that rectangle: `half_w`/`half_h` are its half-extents and `cx`/`cy`
// its centre. Uniform fields arrive as `vec4`, hence `u.half_w.x`.

// Half-height of the fallback rect, in world units.
//
// `View` belongs to the control layer, and a component with no entity reads
// as zeros — which would divide the world by nothing and put every quad at
// infinity. A stage that shows nothing looks exactly like a broken shader, so
// an undescribed view falls back to a screen-shaped rect instead of to NaN.
// Once `game/game3.rs` spawns a `View`, this is never reached.
const VN_FALLBACK_HALF_H : f32 = 1.0;

fn vn_half() -> vec2<f32> {
    if (u.half_w.x > 1e-4 && u.half_h.x > 1e-4) {
        return vec2<f32>(u.half_w.x, u.half_h.x);
    }
    let aspect = se.resolution.x / max(se.resolution.y, 1.0);
    return vec2<f32>(VN_FALLBACK_HALF_H * aspect, VN_FALLBACK_HALF_H);
}

fn vn_centre() -> vec2<f32> {
    return vec2<f32>(u.cx.x, u.cy.x);
}

// World units per screen pixel — the width an edge has to be feathered over
// for it to land on about one pixel however far the view is zoomed.
fn vn_texel() -> f32 {
    return 2.0 * vn_half().y / max(se.resolution.y, 1.0);
}

// game4's look. `SpriteInstance` and `View` come from contract.wgsl, which the
// host prepends: the instance input *is* the component record.
//
// Every sprite is a quad around its centre. `uv.x` is the shape id and the
// fragment stage cuts the quad down to it, so one draw paints a ship, a bee, a
// bullet, a bomb, a star and a bar:
//   0 ship   1 bullet   2 bee   3 bomb   4 star   5 bar

@group(0) @binding(0) var<uniform> view: View;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) tint: vec4<f32>,
    @location(1) p: vec2<f32>,
    @location(2) @interpolate(flat) shape: f32,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, inst: SpriteInstance) -> VsOut {
    var corner = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.5), vec2<f32>( 0.5, -0.5), vec2<f32>( 0.5,  0.5),
        vec2<f32>(-0.5, -0.5), vec2<f32>( 0.5,  0.5), vec2<f32>(-0.5,  0.5),
    );
    let c = corner[vi];
    let world = inst.pos + c * inst.size;

    // Orthographic: the visible box is half-extents around a centre.
    let ndc = vec2<f32>(
        (world.x - view.cx) / max(view.half_w, 0.0001),
        (world.y - view.cy) / max(view.half_h, 0.0001),
    );

    var o: VsOut;
    o.clip = vec4<f32>(ndc, 0.0, 1.0);
    o.tint = inst.tint;
    o.p = c;
    o.shape = inst.uv.x;
    return o;
}

@fragment
fn fs_main(i: VsOut) -> @location(0) vec4<f32> {
    let p = i.p;
    var c = i.tint.rgb;
    var a = 0.0;

    if i.shape < 0.5 {
        // Ship: a dart. The half-width shrinks toward the nose, with a cockpit.
        let hw = (0.5 - p.y) * 0.5;
        a = select(0.0, 1.0, abs(p.x) <= hw);
        let cockpit = select(0.0, 1.0, length(p - vec2<f32>(0.0, -0.05)) < 0.11);
        c = mix(c, vec3<f32>(1.0, 1.0, 1.0), cockpit * 0.85);
    } else if i.shape < 1.5 {
        // Bullet: a capsule, thin and bright.
        a = 1.0 - smoothstep(0.30, 0.5, abs(p.x));
    } else if i.shape < 2.5 {
        // Bee: a striped body, two wings, two eyes.
        let body = select(0.0, 1.0, length(vec2<f32>(p.x / 0.26, p.y / 0.36)) < 1.0);
        let wing = select(0.0, 1.0, length(vec2<f32>((abs(p.x) - 0.30) / 0.20, (p.y - 0.14) / 0.15)) < 1.0);
        let stripe = select(0.0, 1.0, sin(p.y * 34.0) > 0.0);
        let eye = select(0.0, 1.0, length(vec2<f32>(abs(p.x) - 0.09, p.y - 0.20)) < 0.045);
        a = max(body, wing);
        c = mix(c, vec3<f32>(0.10, 0.08, 0.14), body * stripe * 0.55);
        c = mix(c, vec3<f32>(0.80, 0.95, 1.0), wing * (1.0 - body) * 0.9);
        c = mix(c, vec3<f32>(0.0, 0.0, 0.0), eye);
    } else if i.shape < 3.5 {
        // Bomb: a ball.
        a = 1.0 - smoothstep(0.34, 0.5, length(p));
    } else if i.shape < 4.5 {
        // Star: a dot.
        a = 1.0 - smoothstep(0.15, 0.5, length(p));
    } else {
        // Bar: solid.
        a = 1.0;
    }

    if a < 0.5 {
        discard;
    }
    return vec4<f32>(c, i.tint.a);
}

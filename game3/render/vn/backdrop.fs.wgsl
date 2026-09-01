// The scene behind everything. Fullscreen, so there is nothing to instance.
//
// `Stage` is this pass's uniform: `fade` is how lit the scene is, `cx`/`cy`
// are the slow drift that keeps a still frame from looking frozen. Both are
// the control layer's to write — this pass has no clock of its own beyond the
// globals, and deliberately no way to reach one.

@fragment
fn fs_main(o : SeVsOut) -> @location(0) vec4<f32> {
    // `SeVsOut.uv` runs y-down; the backdrop reads as a sky, so flip it.
    let p = vec2<f32>(o.uv.x, 1.0 - o.uv.y);

    // Where the light is coming from, drifted by the control layer. Held
    // inside the frame so a runaway value cannot push the glow off-screen.
    let c = vec2<f32>(0.5 + clamp(u.cx.x, -0.4, 0.4),
                      0.35 + clamp(u.cy.x, -0.3, 0.3));

    var col = mix(INK, DUSK, smoothstep(-0.15, 1.0, p.y));

    // A single warm source low in the frame, squashed vertically so it reads
    // as a horizon rather than a lamp.
    let d = length((p - c) * vec2<f32>(1.0, 1.7));
    let glow = exp(-d * d * 5.0);
    col += HAZE * glow * 0.60 + EMBER * glow * glow * 0.35;

    // Strata, so the upper half is never one flat colour.
    let strata = sin(p.y * 21.0 + u.cx.x * 4.0) * sin(p.x * 3.0 - u.cy.x * 2.0);
    col += DUSK * 0.22 * strata * smoothstep(0.25, 0.95, p.y);

    // Vignette, then the fade. `Stage.fade` is 0 = black, 1 = fully lit, so
    // it multiplies the whole backdrop and nothing here second-guesses it.
    col *= 1.0 - 0.55 * length((p - vec2<f32>(0.5, 0.5)) * vec2<f32>(1.1, 1.0));
    return vec4<f32>(max(col, vec3<f32>(0.0)) * clamp(u.fade.x, 0.0, 1.0), 1.0);
}

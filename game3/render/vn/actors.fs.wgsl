// Shaping a rectangle into somebody standing there.
//
// The quad arrives as a rectangle and has to leave as a figure, and the only
// place that can happen is here: there is no font, no sprite and no portrait
// asset to sample, so the silhouette is a distance field. Swapping the asset
// module changes who is on stage; it does not change this, which is the split
// the design asks for.

@fragment
fn fs_main(o : VnOut) -> @location(0) vec4<f32> {
    let p = o.quad;
    let w = max(o.fuzz * 1.5, 0.004);

    // A head over a body, unioned — enough shape that two actors side by side
    // read as two people rather than two blocks.
    let body = sd_round_box(p - vec2<f32>(0.0, -0.35), vec2<f32>(0.70, 0.64), 0.42);
    let head = sd_round_box(p - vec2<f32>(0.0,  0.62), vec2<f32>(0.34, 0.34), 0.32);
    let d = min(body, head);

    // Hue by slot, so the cast is distinguishable without the renderer being
    // told a single name.
    let hue = fract(o.slot * 0.37 + 0.11);
    var col = mix(HAZE, EMBER, hue) * 0.75 + DUSK * 0.5;

    // Lit from the same low, warm side as the backdrop's glow.
    col *= 0.62 + 0.38 * clamp(0.55 - p.x * 0.45 - p.y * 0.18, 0.0, 1.0);

    // A rim just inside the silhouette, so an actor dimmed by `tint` still
    // has an outline and "who is not speaking" stays readable.
    col += EDGE * band(abs(d + 0.05), 0.045) * 0.55;

    // `Actor.tint` is the control layer's, and dimming everyone but
    // `Line.speaker` is the whole of "who is talking" as far as this pass is
    // concerned — it multiplies, and this pass never overrides it.
    return vec4<f32>(col * o.tint.rgb, band(d, w) * o.tint.a);
}

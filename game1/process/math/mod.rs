//! Trigonometry, shared across `process/`.
//!
//! A folder with no sibling `.rs` is shared by every module in its category —
//! which is the only way two stages can share code, because each stage is
//! compiled as its own crate. `orbit.rs` cannot reach into `spin.rs`, and that
//! is the point: a module is replaceable precisely because nothing links to it.
//!
//! A module is `no_std` and imports nothing, so it brings its own maths.

const TAU: f32 = 6.283_185_5;
const PI: f32 = 3.141_592_7;

/// Bhaskara's approximation, folded to the full circle. About 0.2% peak
/// error — a fraction of a pixel at any scale a body is drawn at, and it costs
/// no library and no import.
pub fn sin(x: f32) -> f32 {
    let mut a = x % TAU;
    if a < 0.0 {
        a += TAU;
    }
    let (a, sign) = if a > PI { (a - PI, -1.0) } else { (a, 1.0) };
    let n = 16.0 * a * (PI - a);
    sign * n / (5.0 * PI * PI - 4.0 * a * (PI - a))
}

pub fn cos(x: f32) -> f32 {
    sin(x + PI * 0.5)
}

/// Two Newton steps from a halved exponent. Enough for a near-unit vector,
/// which is all a rotation axis ever is.
pub fn sqrt(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut g = f32::from_bits((x.to_bits() >> 1) + 0x1fc0_0000);
    g = 0.5 * (g + x / g);
    0.5 * (g + x / g)
}

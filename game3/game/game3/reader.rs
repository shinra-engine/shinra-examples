//! The reader: a typewriter and a line cursor, as a pure function.
//!
//! Private to `game/game3.rs`, like `parse.rs`. It takes the cursor, the time
//! step, whether the player pressed advance, and the script, and returns the
//! next cursor. No clock reads, no statics, no engine types — so the same
//! inputs always give the same cursor, and `rustc --test` can build it alone.
//!
//! One press does one thing: while a line is still being typed it finishes the
//! reveal, and only once the line is fully shown does it move to the next one.
//! A press therefore never skips a line the player has not read.

#[cfg(not(test))]
use super::parse;
#[cfg(test)]
#[path = "parse.rs"]
mod parse;

/// Characters revealed per second.
pub const CPS: f32 = 32.0;
/// What fits in the box: 43 columns by 3 rows.
pub const MAX_GLYPHS: u32 = 129;
/// Length of the fallback line shown when a script has no `say` lines.
const FALLBACK_LEN: u32 = 17;

#[derive(Clone, Copy, Default)]
pub struct Cursor {
    pub index: u32,
    pub revealed: u32,
    pub speaker: u32,
    pub done: u32,
    pub len: u32,
    pub elapsed: f32,
}

/// Fill `len` and `speaker` from the line under the cursor. A script with no
/// lines is valid: it gets the fallback line, spoken by slot 0.
pub fn describe(c: &mut Cursor, s: &parse::Script<'_>) {
    match s.line(c.index) {
        Some(say) => {
            c.len = (say.text.len() as u32).min(MAX_GLYPHS);
            c.speaker = say.speaker;
        }
        None => {
            c.len = FALLBACK_LEN;
            c.speaker = 0;
        }
    }
}

pub fn step(c: &mut Cursor, dt: f32, advance: bool, s: &parse::Script<'_>) {
    let full = c.len as f32 / CPS;
    if advance && c.done != 0 {
        c.index = c.index.wrapping_add(1);
        c.revealed = 0;
        c.elapsed = 0.0;
        c.done = 0;
        describe(c, s);
    } else if advance {
        c.elapsed = full;
    } else {
        c.elapsed = (c.elapsed + dt).min(full);
    }
    c.revealed = ((c.elapsed * CPS) as u32).min(c.len);
    c.done = (c.revealed >= c.len) as u32;
}

pub fn restart(c: &mut Cursor, s: &parse::Script<'_>) {
    c.index = 0;
    c.elapsed = 0.0;
    c.revealed = 0;
    c.done = 0;
    describe(c, s);
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO: &[u8] = b"say a \"0123456789abcdef\"\nsay b \"xy\"\n";

    fn fresh(s: &parse::Script<'_>) -> Cursor {
        let mut c = Cursor::default();
        restart(&mut c, s);
        c
    }

    #[test]
    fn typewriter_reaches_len_after_len_over_cps() {
        let s = parse::Script::new(TWO);
        let mut c = fresh(&s);
        assert_eq!(c.len, 16);
        step(&mut c, 0.25, false, &s);
        assert_eq!(c.revealed, 8);
        assert_eq!(c.done, 0);
        step(&mut c, 0.25, false, &s);
        assert_eq!((c.revealed, c.done), (16, 1));
        step(&mut c, 5.0, false, &s);
        assert_eq!((c.revealed, c.done, c.index), (16, 1, 0));
    }

    #[test]
    fn first_press_finishes_second_advances() {
        let s = parse::Script::new(TWO);
        let mut c = fresh(&s);
        step(&mut c, 0.0, true, &s);
        assert_eq!((c.index, c.revealed, c.done), (0, 16, 1));
        step(&mut c, 0.0, true, &s);
        assert_eq!((c.index, c.revealed, c.done, c.len, c.speaker), (1, 0, 0, 2, 1));
    }

    #[test]
    fn index_wraps_at_the_last_line() {
        let s = parse::Script::new(TWO);
        let mut c = fresh(&s);
        for _ in 0..4 {
            step(&mut c, 0.0, true, &s);
        }
        assert_eq!((c.index, c.speaker, c.len), (2, 0, 16));
    }

    #[test]
    fn describe_on_an_empty_script_gives_len_17() {
        let s = parse::Script::new(b"");
        let mut c = Cursor::default();
        describe(&mut c, &s);
        assert_eq!((c.len, c.speaker), (17, 0));
        step(&mut c, 1.0, false, &s);
        assert_eq!(c.done, 1);
        step(&mut c, 0.0, true, &s);
        assert_eq!((c.index, c.len), (1, 17));
    }

    #[test]
    fn long_lines_are_clamped_to_the_box() {
        let src = format!("say a \"{}\"\n", "x".repeat(300));
        let s = parse::Script::new(src.as_bytes());
        assert_eq!(fresh(&s).len, MAX_GLYPHS);
    }

    #[test]
    fn restart_resets() {
        let s = parse::Script::new(TWO);
        let mut c = fresh(&s);
        step(&mut c, 0.0, true, &s);
        step(&mut c, 0.0, true, &s);
        restart(&mut c, &s);
        assert_eq!((c.index, c.revealed, c.done, c.len, c.speaker), (0, 0, 0, 16, 0));
        assert_eq!(c.elapsed, 0.0);
    }
}

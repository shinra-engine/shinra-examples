//! The typewriter, and the only thing that moves the reader forward.
//!
//! # How parsed text reaches this stage
//!
//! It does not. A stage cannot read an asset, so it never sees a byte of
//! `story.rpy`. What it sees is `Line`, and the split of that component is
//! the whole answer:
//!
//! ```text
//!   asset/act*.so   story.rpy as bytes
//!         |                                    (Ctl::asset)
//!   game/game3.rs   parses once on start, keeps the parsed script,
//!         |          and *describes the current line* into the world:
//!         |              Line.len     characters in line `index`
//!         |              Line.speaker which Actor.slot is talking
//!         |          then spawns one Glyph per revealed character
//!         v
//!   process/script.rs   turns dt and a key press into Line.revealed
//!         |                                     and Line.index
//!         v
//!   render/vn.rs        draws the glyphs that exist
//! ```
//!
//! So this file knows *how long* the current line is and nothing about what
//! it says. That is deliberate: the act can be swapped for another with a
//! different cast and a different script, and this stage is not rebuilt and
//! cannot tell.
//!
//! # The handshake: `len == 0` means "describe this line"
//!
//! Only the stage sees the advance key, so only the stage can move `index`.
//! Only control can read the script, so only control can say how long the new
//! line is. Neither can call the other, so they meet in the data: when this
//! stage advances, it sets `len = 0`, which is a request. Control runs at the
//! top of the next tick (control, then stages, then the graph), sees a line
//! it has not described, and fills in `len` and `speaker` and reconciles the
//! glyph roster. Until it does, `len == 0` and this stage does nothing —
//! there is no line to type out yet. A blank line is not a thing the parser
//! emits, so zero is free to carry that meaning.
//!
//! The field split that follows from it, and the one `game/game3.rs` must
//! honour: control writes `speaker` and `len`; this stage writes `index`,
//! `revealed`, `elapsed` and `done`. One writer each, as always.
//!
//! End of script is control's call, not this stage's — the stage only counts
//! up, because it does not know how many lines there are. Control sees an
//! `index` past the last line and decides whether that holds, wraps, or ends
//! the act.

use data::Line;

/// Characters per second. A visual novel's reading speed is a game number, so
/// it lives in the game, next to the loop that spends it.
const CPS: f32 = 32.0;

/// Advance the line. Two keys because a reader reaches for either.
const ADVANCE: [u8; 2] = [se::key::SPACE, se::key::ENTER];

/// Type out the current line, and move to the next one when asked.
///
/// One press does one thing: mid-reveal it finishes the reveal, and only once
/// the line stands complete does another press move on. That is why `done` is
/// read from the *previous* frame here and written at the end — a single
/// press can never skip a line unread.
#[se::stage]
fn script(l: &mut Line, i: &se::Input, dt: f32) {
    // Control has not described this line yet. Nothing to reveal, and
    // advancing now would skip a line nobody has seen.
    if l.len == 0 {
        return;
    }

    let full = l.len as f32 / CPS;
    let advance = ADVANCE.iter().any(|&k| i.just_pressed(k));

    if advance && l.done != 0 {
        l.index += 1;
        l.revealed = 0;
        l.elapsed = 0.0;
        l.done = 0;
        // The request. Control answers it before this stage runs again.
        l.len = 0;
        return;
    }

    // Clamped rather than left to climb, so a line left standing for an hour
    // does not lose `dt` into the far end of an f32.
    l.elapsed = if advance { full } else { (l.elapsed + dt).min(full) };

    // A float-to-int cast saturates in Rust, so a stall that hands us a huge
    // `dt` reveals the whole line instead of wrapping to none of it.
    l.revealed = ((l.elapsed * CPS) as u32).min(l.len);
    l.done = u32::from(l.revealed >= l.len);
}

se::stages!(script);

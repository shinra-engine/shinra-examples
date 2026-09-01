//! Control for game3: the only station that knows the time.
//!
//! # What this file is, and what it is not yet
//!
//! It is the smallest control that makes game3 run. There is no parser here:
//! the line it shows is the `const` below, not a byte of `story.rpy`. Reading
//! the asset and turning it into a script is the next ticket's job, and it
//! slots in at exactly one place — `describe`, which is the only function
//! that decides what the current line says. Everything around it is already
//! written against "a line has a speaker and a length", which is all the rest
//! of the bundle ever agreed to know.
//!
//! # The `len == 0` handshake, from this side
//!
//! `process/script.rs` fixed the field ownership and this file honours it:
//!
//! ```text
//!   control (here)             writes  Line.speaker, Line.len
//!   stage   (process/script)   writes  Line.index, revealed, elapsed, done
//! ```
//!
//! Only the stage sees the advance key, so only the stage moves `index`; only
//! control can reach an asset, so only control can say how long a line is.
//! They meet in the data. When the stage advances it sets `len = 0` as a
//! request; control runs at the top of the next tick, before any stage, sees
//! a line nobody has described, and fills in `speaker` and `len`. While
//! `len == 0` the stage does nothing, so no line is ever typed out before it
//! has been described.
//!
//! `Ctl::set` writes a whole component, so describing a line means reading
//! `Line`, changing this file's two fields and writing it back. That is safe
//! precisely because control runs first: the stage-owned fields written back
//! are the ones the stage itself left there, untouched, and the stage will
//! overwrite them later in the same frame.
//!
//! # Glyphs
//!
//! A stage cannot spawn, so the text roster is control's. `bundle/data.rs`
//! says only the *revealed prefix* is spawned — that is what makes the
//! typewriter visible without the text pass ever learning that `Line` exists,
//! so the roster is reconciled against `Line.revealed` every tick rather than
//! against `len` once per line. Reconciled, not spawned: `start` runs again
//! after a hot reload, and every function here compares what should exist
//! against what does.
//!
//! The character code goes in `Glyph.uv[0]` as an `f32`, which is where the
//! text pass reads it. There is no glyph pass yet; the roster is correct
//! before anything draws it, which is the order that lets that ticket see its
//! own output.

// Private to this file: `game/game3/` is named after it.
mod parse;

use data::{Actor, Glyph, Line, Stage, View};
use se::{Ctl, Entity, Frame, Slot};

/// The asset every act publishes. Which module answers to it is the asset
/// slot's business, not this file's — that is the whole point.
const STORY: &str = "story.rpy";

/// The acts this game can point its asset slot at. Adding one means adding an
/// `asset/*.rs` and a name here, and nothing else in the bundle changes.
const ACTS: [&str; 2] = ["act1", "act2"];

/// Shown when the asset is missing or says nothing. A story file is content
/// and content must not be able to leave the screen broken.
const FALLBACK: &str = "(no story loaded)";
/// A line longer than this is clipped rather than allowed to eat the world.
const MAX_GLYPHS: usize = 256;

// --- the view rectangle -----------------------------------------------------
// A visual novel is a flat stage, so `View` is just the rectangle of world the
// screen shows. Roughly 16:9, so a shot at 100x56 is not stretched.
const HALF_W: f32 = 1.60;
const HALF_H: f32 = 0.90;

// --- where the text goes ----------------------------------------------------
// These mirror the panel `render/vn/box.fs.wgsl` draws, in the same
// view-relative units, because that file said what it owed the glyphs and this
// is the side that owes it back. They are fractions of the view half-extents.
const BOX_BOTTOM: f32 = 0.62;
const BOX_HALF_W: f32 = 0.90;
const BOX_HALF_H: f32 = 0.26;
/// Inset from the panel edge to the first character, in world units.
const PAD: f32 = 0.10;
/// Advance per character and per row, in world units.
const ADVANCE: f32 = 0.062;
const LEADING: f32 = 0.130;
/// Half-extent of one glyph quad.
const GLYPH_HALF: [f32; 2] = [0.024, 0.045];

/// Called once the slots are bound, and again after every hot reload — so it
/// reconciles. Running it twice must be a no-op, which is the whole reason
/// none of these functions spawn without looking first.
fn start(c: &mut Ctl) {
    ensure_view(c);
    ensure_stage(c);
    ensure_line(c);
    let bytes = c.asset(STORY).unwrap_or(&[]);
    let script = parse::parse(bytes);
    reconcile_cast(c, script.cast, 0, 1.0);
    c.log("game3: stage reconciled");
}

fn tick(c: &mut Ctl, f: &Frame) {
    let input = unsafe { f.input() };

    // Read and parse the act every tick rather than caching it. A `.so` loses
    // its statics when it is swapped, so a cached script would be a lie the
    // moment the asset slot moved — and parsing a few kilobytes is cheaper
    // than the machinery for keeping a cache honest. It also means swapping
    // the act takes effect with no invalidation step at all.
    let bytes = c.asset(STORY).unwrap_or(&[]);
    let script = parse::parse(bytes);

    // A reload can land between ticks, and `start` may not have run against
    // this world yet. Reconciling here too costs one query when everything is
    // already in place.
    let stage = ensure_stage(c);
    let line_e = ensure_line(c);
    ensure_view(c);

    let Some(mut line) = c.get::<Line>(line_e, "Line") else { return };

    // The handshake. `len == 0` is the stage asking what this line says.
    if line.len == 0 {
        describe(&mut line, &script);
        c.set(line_e, "Line", &line);
        // A new line starts with nothing revealed, so the previous line's
        // glyphs are not left standing under it.
        reconcile_glyphs(c, 0, "");
    }

    // Only the prefix the typewriter has reached exists. `revealed` is the
    // stage's, and clamping to the line we described keeps a roster that is
    // one frame stale from running off the end of it.
    let shown = (line.revealed as usize).min(line.len as usize);
    reconcile_glyphs(c, shown, text_of(&script, line.index));

    // Who is talking, and how lit the scene is. Both multiply into
    // `Actor.tint`, which is the only thing `render/vn.rs` reads about either.
    // The cast size comes from the act: a definition it carries, discovered by
    // the parser, turned into entities here — content never spawns its own.
    reconcile_cast(c, script.cast, line.speaker, stage.1.fade);

    // The backdrop's slow drift. `game.so` is the only station that knows the
    // time, so a shader that wants one gets it as a number in the world.
    let t = f.t as f32;
    let mut s = stage.1;
    s.cx = (t * 0.11).sin() * 0.22;
    s.cy = (t * 0.07).cos() * 0.10;
    c.set(stage.0, "Stage", &s);

    // Swap the act. This is the design's claim made literal: a different
    // `asset/*.so` answers to `story.rpy`, and no other module is rebuilt or
    // even told. The new bytes are a *definition* change, so the roster has to
    // be reconciled against them — which is control's job and nobody else's,
    // and happens on the next tick by way of the `len == 0` request below.
    if input.just_pressed(b'R') {
        let next = (s.act as usize + 1) % ACTS.len();
        s.act = next as u32;
        c.set(stage.0, "Stage", &s);
        c.set_slot(Slot::Asset, ACTS[next]);

        // Start the new act at its first line. Leaving `index` where it was
        // would point into a script that no longer has that many lines.
        line.index = 0;
        line.revealed = 0;
        line.elapsed = 0.0;
        line.done = 0;
        line.len = 0;
        c.set(line_e, "Line", &line);
        c.log("game3: act swapped");
    }

    // GameTok: a swipe is not this game's business beyond passing it on.
    if input.swipe != 0 {
        c.swipe(input.swipe as i32);
    }
}

/// What the current line says, now answered from the act rather than a
/// constant. `index` was always the question; the script is the answer.
fn describe(line: &mut Line, script: &parse::Script) {
    match script.line(line.index) {
        Some(say) => {
            line.speaker = say.speaker;
            line.len = say.text.len().min(MAX_GLYPHS) as u32;
        }
        None => {
            line.speaker = 0;
            line.len = FALLBACK.len() as u32;
        }
    }
}

/// The text of the current line. Everything that lays out glyphs asks this, so
/// there is one place that decides what "the current line" means.
fn text_of<'a>(script: &'a parse::Script<'a>, index: u32) -> &'a str {
    script.line(index).map(|s| s.text).unwrap_or(FALLBACK)
}

// ---------------------------------------------------------------------------
// reconcile: what should exist, against what does
// ---------------------------------------------------------------------------

/// Lay `want` characters of `LINE` out inside the dialogue box, left to right
/// and wrapping down.
///
/// Every surviving glyph is rewritten, not just the new ones, so the roster
/// never depends on `query` handing entities back in the order they were
/// spawned — glyph *k* of the answer is character *k*, whichever entity that
/// turns out to be.
fn reconcile_glyphs(c: &mut Ctl, want: usize, line_text: &str) {
    let text = line_text.as_bytes();
    let want = want.min(text.len()).min(MAX_GLYPHS);

    let mut have = [0u64; MAX_GLYPHS];
    let n = (c.query("Glyph", &mut have) as usize).min(MAX_GLYPHS);

    for k in want..n {
        c.despawn(have[k]);
    }
    for slot in have.iter_mut().take(want).skip(n) {
        *slot = c.spawn();
    }
    for (k, &ch) in text.iter().enumerate().take(want) {
        c.set(have[k], "Glyph", &glyph(k, ch));
    }
}

/// One character's quad. Wrapping is by cell count rather than by word: there
/// is one line and it is a constant, so a line breaker is state this ticket
/// does not have to carry.
fn glyph(k: usize, ch: u8) -> Glyph {
    let left = -HALF_W * BOX_HALF_W + PAD;
    let top = -HALF_H * BOX_BOTTOM + HALF_H * BOX_HALF_H - PAD;
    let cols = (((HALF_W * BOX_HALF_W - PAD) * 2.0) / ADVANCE) as usize;
    let cols = cols.max(1);

    let (col, row) = (k % cols, k / cols);
    Glyph {
        pos: [
            left + (col as f32 + 0.5) * ADVANCE,
            top - (row as f32 + 0.5) * LEADING,
        ],
        size: GLYPH_HALF,
        // The character code, where the text pass reads it. The rest is the
        // cell it would sample if there were an atlas to sample.
        uv: [ch as f32, 0.0, 1.0, 1.0],
    }
}

/// The cast. `speaker` is lit and everyone else is dimmed, and `fade`
/// multiplies through both — that is the whole of "who is talking" as far as
/// `render/` is concerned.
fn reconcile_cast(c: &mut Ctl, want: u32, speaker: u32, fade: f32) {
    let mut have = [0u64; 16];
    let n = c.query("Actor", &mut have).min(have.len() as u32);

    for k in want..n {
        c.despawn(have[k as usize]);
    }
    for k in n..want.min(have.len() as u32) {
        have[k as usize] = c.spawn();
    }
    for k in 0..want.min(have.len() as u32) {
        // Spread across the stage, standing with their feet behind the box.
        let t = (k as f32 + 0.5) / want.max(1) as f32;
        let lit = if k == speaker { 1.0 } else { 0.45 };
        c.set(
            have[k as usize],
            "Actor",
            &Actor {
                slot: k,
                tint: [lit * fade, lit * fade, lit * fade, fade.clamp(0.0, 1.0)],
                pos: [(t * 2.0 - 1.0) * HALF_W * 0.62, 0.05],
                size: [0.34, 0.52],
            },
        );
    }
}

fn ensure_view(c: &mut Ctl) -> View {
    if let Some((_, v)) = first::<View>(c, "View") {
        return v;
    }
    let v = View { half_w: HALF_W, half_h: HALF_H, cx: 0.0, cy: 0.0 };
    let e = c.spawn();
    c.set(e, "View", &v);
    v
}

fn ensure_stage(c: &mut Ctl) -> (Entity, Stage) {
    if let Some(found) = first::<Stage>(c, "Stage") {
        return found;
    }
    let s = Stage { fade: 1.0, act: 0, cx: 0.0, cy: 0.0 };
    let e = c.spawn();
    c.set(e, "Stage", &s);
    (e, s)
}

/// The cursor. It carries `Line::START`, whose `len` is zero — which is the
/// handshake's opening move: the first tick describes line zero.
fn ensure_line(c: &mut Ctl) -> Entity {
    if let Some((e, _)) = first::<Line>(c, "Line") {
        return e;
    }
    let e = c.spawn();
    c.set(e, "Line", &Line::START);
    e
}

fn first<T: Copy>(c: &mut Ctl, name: &'static str) -> Option<(Entity, T)> {
    let mut buf = [0u64; 4];
    if c.query(name, &mut buf) == 0 {
        return None;
    }
    c.get::<T>(buf[0], name).map(|v| (buf[0], v))
}

se::control! {
    name: "vn",
    slots: [
        se::SlotBind::new(Slot::Asset, "act1"),
        se::SlotBind::new(Slot::Render, "vn"),
    ],
    start: start,
    tick: tick,
}

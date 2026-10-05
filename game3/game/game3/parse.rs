//! The `.rpy` dialect, as a pure function over bytes.
//!
//! Private to `game/game3.rs` — this folder is named after that file, so
//! nothing else in `game/` can reach it. That is the right place for it: a
//! parser is content interpretation, and content is control's business alone.
//! `process/` never sees a byte of the script, and the engine has no idea
//! dialogue exists.
//!
//! Nothing here touches `Ctl`, spawns anything, or knows what a component is.
//! It takes bytes and returns what they say, which is what makes it the one
//! part of game3 that can be reasoned about without the engine in the room.
//!
//! # The dialect
//!
//! Documented at the head of `asset/act1/story.rpy`, which is the file this
//! exists to read:
//!
//! ```text
//!   # ...                    comment
//!   label <name>             a jump target
//!   scene <name>             the backdrop
//!   show <actor>             put an actor on stage
//!   hide <actor>             take one off
//!   say <actor> "<text>"     one line of dialogue
//!   choice / -> / jump       branching
//! ```
//!
//! Unknown lines are skipped rather than fatal. A story file is content, and
//! content must not be able to crash the game — an act with a keyword this
//! parser has not learned should lose that line, not the player's evening.

/// Roster ceiling. A name's slot never exceeds `MAX_CAST - 1`; further names
/// share the last slot rather than overflow the fixed table.
pub const MAX_CAST: usize = 16;

/// One line of dialogue.
pub struct Say<'a> {
    /// The `Actor.slot` assigned to this speaker.
    pub speaker: u32,
    pub text: &'a str,
}

/// An act, borrowed from its bytes. Nothing is copied or stored: every query
/// re-reads the text, which costs O(bytes) and keeps slots stable by
/// construction — the same bytes always give the same slot.
pub struct Script<'a> {
    text: &'a str,
}

impl<'a> Script<'a> {
    /// Invalid UTF-8 truncates at the last valid byte: a broken asset yields
    /// fewer lines, never a panic.
    pub fn new(bytes: &'a [u8]) -> Script<'a> {
        let text = match core::str::from_utf8(bytes) {
            Ok(t) => t,
            Err(e) => core::str::from_utf8(&bytes[..e.valid_up_to()]).unwrap_or(""),
        };
        Script { text }
    }

    /// The `say` lines in order, each with its speaker's slot.
    pub fn says(&self) -> Says<'a> {
        Says { lines: self.text.lines(), names: [""; MAX_CAST], n: 0 }
    }

    /// Number of `say` lines.
    pub fn count(&self) -> u32 {
        self.says().count() as u32
    }

    /// The line at `i`, wrapping. The act loops rather than ending, because
    /// what happens after the last line is a story decision and this act does
    /// not make one.
    pub fn line(&self, i: u32) -> Option<Say<'a>> {
        let n = self.count();
        if n == 0 {
            return None;
        }
        self.says().nth((i % n) as usize)
    }

    /// Distinct actors named by `show`, `hide` or `say`: the roster size,
    /// discovered rather than declared. At least 1, at most `MAX_CAST`.
    pub fn cast(&self) -> u32 {
        let mut it = self.says();
        while it.next().is_some() {}
        (it.n as u32).max(1)
    }
}

/// Iterator over `say` lines. Carries the cast table so slots follow first
/// appearance, including `show`/`hide` lines that precede the speaker.
pub struct Says<'a> {
    lines: core::str::Lines<'a>,
    names: [&'a str; MAX_CAST],
    n: usize,
}

impl<'a> Says<'a> {
    /// A name's index *is* its `Actor.slot`; names past the table clamp.
    fn slot_of(&mut self, name: &'a str) -> u32 {
        if let Some(i) = self.names[..self.n].iter().position(|x| *x == name) {
            return i as u32;
        }
        if self.n == MAX_CAST {
            return (MAX_CAST - 1) as u32;
        }
        self.names[self.n] = name;
        self.n += 1;
        (self.n - 1) as u32
    }
}

impl<'a> Iterator for Says<'a> {
    type Item = Say<'a>;

    fn next(&mut self) -> Option<Say<'a>> {
        while let Some(raw) = self.lines.next() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (kw, rest) = head(line);
            match kw {
                // `say <actor> "<text>"`. Both halves are required; a `say`
                // with no quoted text is malformed and dropped.
                "say" => {
                    let (who, tail) = head(rest);
                    if who.is_empty() {
                        continue;
                    }
                    let Some(text) = quoted(tail) else { continue };
                    let speaker = self.slot_of(who);
                    return Some(Say { speaker, text });
                }
                // `show` puts an actor on stage, so it defines a slot even for
                // someone who never speaks.
                "show" | "hide" => {
                    let (who, _) = head(rest);
                    if !who.is_empty() {
                        self.slot_of(who);
                    }
                }
                // Unknown or not-yet-used keywords are skipped, never fatal.
                _ => {}
            }
        }
        None
    }
}

/// Split a line into its keyword and the rest.
fn head(line: &str) -> (&str, &str) {
    let line = line.trim();
    match line.find(char::is_whitespace) {
        Some(i) => (&line[..i], line[i..].trim()),
        None => (line, ""),
    }
}

/// The text between the first pair of double quotes, if any.
///
/// No escape handling: the dialect has none, and inventing one here would let
/// a story file disagree with the file that documents it.
fn quoted(s: &str) -> Option<&str> {
    let a = s.find('"')?;
    let rest = &s[a + 1..];
    let b = rest.find('"')?;
    Some(&rest[..b])
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACT1: &[u8] = include_bytes!("../../asset/act1/story.rpy");
    const ACT2: &[u8] = include_bytes!("../../asset/act2/story.rpy");

    #[test]
    fn reads_say_lines_and_assigns_slots_in_first_appearance_order() {
        let s = Script::new(b"show jessie\nsay jessie \"hi\"\nsay barret \"go\"\nsay jessie \"no\"\n");
        assert_eq!(s.count(), 3);
        let v: [u32; 3] = [0, 1, 2].map(|i| s.line(i).unwrap().speaker);
        assert_eq!(v, [0, 1, 0]);
        assert_eq!(s.cast(), 2);
    }

    #[test]
    fn comments_blanks_and_unknown_keywords_are_skipped_not_fatal() {
        let s = Script::new(b"# note\n\nwiggle foo\nlabel start\nsay a \"x\"\n");
        assert_eq!(s.count(), 1);
        assert_eq!(s.line(0).unwrap().text, "x");
    }

    #[test]
    fn a_malformed_say_is_dropped_rather_than_guessed() {
        let s = Script::new(b"say jessie\nsay \"orphan\"\n");
        assert_eq!(s.count(), 0);
        assert!(s.line(0).is_none());
        assert_eq!(s.cast(), 1);
    }

    #[test]
    fn line_wraps_so_an_act_loops() {
        let s = Script::new(b"say a \"one\"\nsay a \"two\"\n");
        assert_eq!(s.line(0).unwrap().text, "one");
        assert_eq!(s.line(3).unwrap().text, "two");
    }

    #[test]
    fn act1_has_three_actors() {
        let s = Script::new(ACT1);
        assert_eq!(s.cast(), 3);
        assert!(s.count() > 0);
    }

    #[test]
    fn act2_has_four_actors() {
        let s = Script::new(ACT2);
        assert_eq!(s.cast(), 4);
        assert!(s.count() > 0);
    }

    #[test]
    fn non_utf8_tail_is_dropped_without_panic() {
        let s = Script::new(b"say a \"ok\"\nsay b \"\xff\xfe\"\n");
        assert_eq!(s.count(), 1);
        assert_eq!(s.line(0).unwrap().text, "ok");
        assert_eq!(Script::new(b"\xff").count(), 0);
    }

    #[test]
    fn names_past_the_table_clamp_to_the_last_slot() {
        let mut src = String::new();
        for i in 0..20 {
            src.push_str(&format!("say n{i} \"x\"\n"));
        }
        let s = Script::new(src.as_bytes());
        assert_eq!(s.cast(), MAX_CAST as u32);
        assert_eq!(s.count(), 20);
        assert_eq!(s.line(15).unwrap().speaker, 15);
        assert_eq!(s.line(19).unwrap().speaker, 15);
    }
}

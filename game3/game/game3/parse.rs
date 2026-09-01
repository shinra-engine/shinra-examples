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

/// One line of dialogue.
pub struct Say<'a> {
    /// The `Actor.slot` assigned to this speaker.
    pub speaker: u32,
    pub text: &'a str,
}

pub struct Script<'a> {
    pub says: Vec<Say<'a>>,
    /// How many distinct actors the act names. This is the roster size, and it
    /// is discovered rather than declared — an act with a fifth character
    /// costs one more spawn and not one line anywhere else.
    pub cast: u32,
}

impl Script<'_> {
    /// The line at `i`, wrapping. The act loops rather than ending, because
    /// what happens after the last line is a story decision and this act does
    /// not make one.
    pub fn line(&self, i: u32) -> Option<&Say<'_>> {
        if self.says.is_empty() {
            None
        } else {
            self.says.get(i as usize % self.says.len())
        }
    }
}

/// Actor names in first-appearance order. A name's index *is* its
/// `Actor.slot`, so slot 0 is whoever speaks or is shown first.
struct Cast<'a> {
    names: Vec<&'a str>,
}

impl<'a> Cast<'a> {
    fn slot_of(&mut self, name: &'a str) -> u32 {
        if let Some(i) = self.names.iter().position(|n| *n == name) {
            return i as u32;
        }
        self.names.push(name);
        (self.names.len() - 1) as u32
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

/// Read an act. Invalid input yields fewer lines, never a panic.
pub fn parse(src: &[u8]) -> Script<'_> {
    // A story file that is not UTF-8 is a broken asset, not a crash.
    let text = match core::str::from_utf8(src) {
        Ok(t) => t,
        Err(e) => core::str::from_utf8(&src[..e.valid_up_to()]).unwrap_or(""),
    };

    let mut cast = Cast { names: Vec::new() };
    let mut says = Vec::new();

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (kw, rest) = head(line);
        match kw {
            "say" => {
                // `say <actor> "<text>"`. Both halves are required; a `say`
                // with no quoted text is malformed and dropped.
                let (who, tail) = head(rest);
                if who.is_empty() {
                    continue;
                }
                let Some(text) = quoted(tail) else { continue };
                let speaker = cast.slot_of(who);
                says.push(Say { speaker, text });
            }
            // `show` is what puts an actor on stage, so it defines a slot even
            // for someone who never speaks.
            "show" | "hide" => {
                let (who, _) = head(rest);
                if !who.is_empty() {
                    cast.slot_of(who);
                }
            }
            // Recognised, and deliberately unused: the dialect has these and
            // this act does not act on them yet. Listing them says "skipped on
            // purpose" rather than letting them fall through as unknown.
            "scene" | "label" | "jump" | "choice" | "->" => {}
            _ => {}
        }
    }

    let n = cast.names.len() as u32;
    Script { says, cast: n.max(1) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_say_lines_and_assigns_slots_in_first_appearance_order() {
        let s = parse(b"show jessie\nsay jessie \"hi\"\nsay barret \"go\"\nsay jessie \"no\"\n");
        assert_eq!(s.len(), 3);
        assert_eq!(s.says[0].speaker, 0);
        assert_eq!(s.says[1].speaker, 1);
        assert_eq!(s.says[2].speaker, 0);
        assert_eq!(s.cast, 2);
    }

    #[test]
    fn comments_blanks_and_unknown_keywords_are_skipped_not_fatal() {
        let s = parse(b"# note\n\nwiggle foo\nlabel start\nsay a \"x\"\n");
        assert_eq!(s.len(), 1);
        assert_eq!(s.says[0].text, "x");
    }

    #[test]
    fn a_malformed_say_is_dropped_rather_than_guessed() {
        assert_eq!(parse(b"say jessie\nsay \"orphan\"\n").len(), 0);
    }

    #[test]
    fn line_wraps_so_an_act_loops() {
        let s = parse(b"say a \"one\"\nsay a \"two\"\n");
        assert_eq!(s.line(0).unwrap().text, "one");
        assert_eq!(s.line(3).unwrap().text, "two");
    }
}

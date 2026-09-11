//! The component half of game3's contract.
//!
//! game3 is a visual novel, so everything that would normally be a parser's
//! local variable — where we are in the script, how much of the line has been
//! typed out, who is on stage — is declared here instead. That is not
//! ceremony: a `.so` loses its `static`s the moment it is hot-swapped, so any
//! state that must outlive a swap has to be data. The script itself is bytes
//! in an `asset/*.so`; the *position* in it lives here.
//!
//! `process/`, `render/` and `game/` all agree about this file and nothing
//! else, which is why swapping the asset module changes the story without a
//! single line of the other three changing.

se::components! {
    /// Where the reader is, and how far the typewriter has got.
    ///
    /// One entity carries this — it is the cursor for the whole game. The control
    /// layer parses the script and owns `index`, `speaker`, `len` and `done`;
    /// `process/script.rs` owns `elapsed` and `revealed`. Splitting it that way
    /// keeps a single writer for every field.
    Line {
        /// Which line of the parsed script is showing.
        index: u32,
        /// How many characters of it have been typed out so far.
        revealed: u32,
        /// Which `Actor.slot` is speaking, so the name plate and the portrait
        /// highlight agree without either of them re-reading the script.
        speaker: u32,
        /// 1 once `revealed` has reached `len` — the reader may advance.
        done: u32,
        /// Characters in this line. Set when the line changes; the reveal stage
        /// clamps against it rather than guessing where the text ends.
        len: u32,
        /// Seconds since this line began. `revealed` is whole characters, so the
        /// fraction has to live somewhere for the typewriter to run at a rate
        /// rather than one character per frame.
        elapsed: f32,
    }

    /// One character of text, as an instance of a unit quad.
    ///
    /// The engine draws meshes; it has no font concept and is not going to get
    /// one. So a glyph is a quad plus a rectangle of the atlas asset, and a line
    /// of dialogue is however many of these entities the control layer decided
    /// should exist. Only the revealed prefix is spawned, which is what makes the
    /// typewriter visible without the text pass needing to know about `Line`.
    Glyph {
        /// Centre, in the ortho units `View` describes.
        pos: [f32; 2],
        /// Half-extent in the same units.
        size: [f32; 2],
        /// Sub-rectangle of the atlas: `[u0, v0, u1, v1]`.
        uv: [f32; 4],
    }

    /// A character standing on stage.
    ///
    /// `slot` is an index into whatever cast the asset module published — a
    /// definition, not a name the renderer knows. An asset carrying a fifth
    /// character therefore costs one more spawn and nothing else.
    Actor {
        slot: u32,
        /// Multiplied into the portrait; dimming everyone but `Line.speaker` is
        /// the whole of "who is talking" as far as `render/` is concerned.
        tint: [f32; 4],
        pos: [f32; 2],
        size: [f32; 2],
    }

    /// The scene around the dialogue.
    ///
    /// A render pass names this as its uniform, so a fade is a number in the
    /// world rather than a timer inside a shader.
    Stage {
        /// 0 = black, 1 = fully lit.
        fade: f32,
        /// Which act is loaded — the index of the asset module in the control
        /// layer's list. Pressing the swap key changes this, and it survives the
        /// reload that follows because it is here and not in a `static`.
        act: u32,
        /// Backdrop centre, for the slow drift that keeps a still frame alive.
        cx: f32,
        cy: f32,
    }

    /// The view. An orthographic camera, since a visual novel is a flat stage.
    ///
    /// Named as a pass uniform exactly the way game1 names `Camera`; plain
    /// scalars, because the host repacks them into the alignment WGSL wants.
    View {
        half_w: f32,
        half_h: f32,
        cx: f32,
        cy: f32,
    }
}

impl Line {
    pub const START: Line = Line {
        index: 0,
        revealed: 0,
        speaker: 0,
        done: 0,
        len: 0,
        elapsed: 0.0,
    };
}

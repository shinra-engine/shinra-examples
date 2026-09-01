# game3 / act1 — "The Reactor Floor"
#
# The whole dialect, and the whole of act one. This file is *data*: it is
# handed to the host as bytes by `asset/act1.rs` and parsed inside game3
# (`process/`, `game/`). The engine has no dialogue concept and is not getting
# one, so nothing below is engine syntax — it is this bundle's private format.
#
#   # ...                    comment. Comments and blank lines are dropped.
#   scene <name>             set the backdrop.
#   show <actor>             put <actor> on stage.
#   hide <actor>             take <actor> off stage.
#   say <actor> "<text>"     one line of dialogue, typed out a character at a
#                            time. `<actor>` names the speaker.
#   choice                   open a menu. Its arms are the `->` lines that
#                            follow it.
#   -> "<text>" <label>      one arm: shown as `<text>`, continues at <label>.
#   label <name>             a jump target.
#   jump <label>             continue at <label>.
#
# Leading whitespace is cosmetic; the arms below are indented only to read.
# Actor names are *definitions*, not names the renderer knows: the parser
# hands each name the next free `Actor.slot` as it first appears, so an act
# with a fifth character costs one more spawn and not one line elsewhere.

label start

scene reactor
show jessie
say jessie "Mako pressure is climbing. Someone left the bleed valve shut."

show barret
say barret "Then open it, and stop narrating."
say jessie "It is welded, Barret. Sixty years of welding."

hide barret
say jessie "...he never does wait for the end of a sentence."

show cloud
say cloud "The valve, or the floor. Pick one."

choice
    -> "Cut the valve." cut
    -> "Vent the floor." vent

label cut
say jessie "Cutting. Stand behind me — this one will spit."
say cloud "They always spit."
jump after

label vent
say jessie "Venting. It will be knee-deep in steam down here for a minute."
say cloud "A minute is a long time on this floor."
jump after

label after
say jessie "Pressure is falling. Whoever welded it wanted it to stay shut."

show barret
say barret "Shinra welds things shut. That is the entire company."
say cloud "Then we came to the right floor."

hide jessie
hide barret
hide cloud
scene black
say cloud "Act one ends where the light does."

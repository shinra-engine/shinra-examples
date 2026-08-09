# shinra

## Layout

- `assets/games/<name>/scene.ron` — game data the TUI scans and cycles
  between (`n` keypress in the viewport).
- `assets/obj/` (meshes), `assets/images/` (sprite sheets),
  `assets/tilesets/`, `assets/scenes/` — shared assets referenced by the games.

This repo holds **no Rust code**. The engine, runner, and TUI live in the
sibling repo `shinra-engine/`. Run the TUI from this directory with
`cargo run -p tui --manifest-path ../shinra-engine/Cargo.toml` so relative
asset paths resolve here.

## Repo hygiene — `.gitignore`

Workers commit with `git add -A`, so anything generated will be staged.
The project root must have a `.gitignore` excluding at least:

- `/target/` — Cargo build output (large; shouldn't appear here anymore but
  the rule stays as a safety net)
- `.tmp/` — local scratch (worker docs, claude-bot output)
- `/.browser/` — Playwright snapshots
- `*.swp`, `.DS_Store`, editor cruft

If a worker finds `.gitignore` missing or incomplete, stop and flag the
ticket as `debugging` — do NOT silently regenerate it inside an unrelated
ticket.

# shinra

## Layout

- `assets/games/<name>/{scene.ron,tscn.ron}` — game data the editor-server
  scans and cycles between (`n` keypress in the viewport).
- `assets/obj/` (meshes), `assets/images/` (sprite sheets),
  `assets/tilesets/`, `assets/scenes/` — shared assets referenced by the games.
- `docker-compose.yml` — runs `shinra-editor-server` (built from
  `../shinra-engine-core`) with this folder mounted at `/game`.

This repo holds **no Rust code**. Engine, runner, editor, and editor-server
all live in the sibling repo `shinra-engine-core/`. The runner there still
expects cdylib `.so` files at `target/debug/libgame*.so`; making it consume
`assets/games/*/scene.ron` is a follow-up in that repo, not here.

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

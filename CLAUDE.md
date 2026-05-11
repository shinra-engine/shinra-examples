# shinra

## Layout

- `games/` — sample cdylib games (game1 bunny, game2 teapot, game3)
- `assets/` — `.obj` meshes loaded by the games
- Engine code lives in the sibling repo `shinra-engine-core/`. Game `Cargo.toml`s
  reach `abi/` and `scene/` via `../../../shinra-engine-core/{abi,scene}` path deps.

## Repo hygiene — `.gitignore`

Workers commit with `git add -A`, so anything generated will be staged.
The project root must have a `.gitignore` excluding at least:

- `/target/` — Cargo build output (large)
- `.tmp/` — local scratch (worker docs, claude-bot output)
- `/.browser/` — Playwright snapshots
- `*.swp`, `.DS_Store`, editor cruft

`Cargo.lock` stays tracked (shinra has binaries).

If a worker finds `.gitignore` missing or incomplete, stop and flag the
ticket as `debugging` — do NOT silently regenerate it inside an unrelated
ticket. Creating `.gitignore` is owned by the ticket that bootstraps the
Cargo manifest, not by per-feature work.

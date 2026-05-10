# shinra

## Layout

- `design.md` — project design notes
- `lattice-cast/` → symlink to `/home/posetmage/download/github/LatticeCast`

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

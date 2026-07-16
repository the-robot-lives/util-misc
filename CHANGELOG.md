# Changelog — utilities/shell/misc-git-utils

## [Unreleased]
- Added PROJ-ARCH.md / PROJ-ARCH.summary.md / PROJ-LAYOUT.md / PROJ-LAYOUT.summary.md under `docs/` (self-documentation of this package's own architecture and layout)

## [m2-doc-pointers-tool] — 2026-07-09 — tag: `utilities-shell-misc-git-utils/m2-doc-pointers-tool`
Introduced `doc-pointers`, a Rust rewrite/addition alongside the shell scripts: scans source for doc-pointer tokens, maintains a JSON pointer DB keyed by deterministic UUIDv5s, and can install a git hook. Landed as a single large addition, then refined twice in quick succession.

### Added
- `doc-pointers` Rust binary (`src/bin/doc-pointers.rs`, ~750 lines) — token-based (U+13000–U+1342F range) doc-pointer scanning, `docs/doc-pointer-db.json` persistence, `--check`/`--write`/`--install-hook` modes
- Cargo packaging (`Cargo.toml`, `Cargo.lock`) and `bin/doc-pointers` launcher wrapper
### Changed
- Infisical- and doc-pointer-related handling reworked (+151 lines) shortly after initial landing
- Further refinement pass on `doc-pointers.rs` (+125/-15 lines) tightening scan/update logic

## [m1-initial-tooling] — 2026-06-14 — tag: `utilities-shell-misc-git-utils/m1-initial-tooling`
Package's initial landing as a subtree: a small set of git convenience shell scripts plus an install `Makefile`.

### Added
- `bin/gcap` — `git commit -a -m <msg> && git push origin HEAD` one-liner
- `bin/gp` — `git push origin HEAD` one-liner
- `bin/submodule-diff`, `bin/submodule-pull` — submodule comparison/sync helpers
- `Makefile` with `compile`/`test`/`install`/`clean` targets (installs `bin/*` to `~/.local/bin`)
### Changed
- `.gitignore` additions for local/dev artifacts

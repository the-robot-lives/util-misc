# Architecture Summary — misc-git-utils

## Overview

Terminal utility package of standalone git helper commands: four bash scripts (`gcap`,
`gp`, `submodule-pull`, `submodule-diff`) plus one Rust binary (`doc-pointers`). No
shared runtime library, no config, no local state — the doc-pointer database lives in
whatever target repo the tool is run against.

## Core Components

- `gcap` / `gp` — one-shot commit-all-and-push / push-HEAD wrappers
- `submodule-pull` — ff-only pull of every `.gitmodules` submodule; skips detached HEADs (tag-aware note)
- `submodule-diff` — recursive nested-submodule diff: staged, unstaged, untracked, path-prefixed
- `doc-pointers` (Rust, single ~1000-line source) — durable cross-document pointers: mints 4-glyph Hieroglyph tokens via UUIDv5, scans repos for `⟦token⟧ Name :: Description` declarations, maintains `docs/doc-pointer-db.json` in the target repo, expands `deeplink:⟦token⟧` Markdown links to `path:line`; `build --check` + pre-commit `hook` gate staleness
- `Makefile` — `test` (cargo fmt/build + `bash -n`), `install` → `~/.local/bin` (release binary, not the cargo-run dev wrapper)

## Key Decisions

- Bash for trivial wrappers; Rust (sole dep: `uuid`) where Unicode/UUID/DB handling is needed
- Pointer identity = token; location resolved on demand by `build`, so links never rot
- Comment-context-only declaration parsing avoids string-literal collisions
- Legacy bare flags (`--write`, `--check`, `--install-hook`) still dispatch to subcommands

## Ecosystem Fit

Installed via the monorepo chain `make install-utilities` → `utilities/shell` `SUBDIRS`
(shared `mk/subdirs.mk`) → local `make install` → `~/.local/bin`. Does not use
`share/k8-lib` and does not read `.infra-config.yaml` — pure git tooling, decoupled from
k8s/deploy conventions.

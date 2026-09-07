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
- `doc-pointers` (Rust, single ~2000-line source) — durable cross-document pointers with four subcommands: `build` (scan declarations, expand `deeplink:` links, `--write`/`--check`), `annotate` (auto-insert markers above public Rust/Elixir/JS fns, dry-run default + closing build), `uuid5` (mint 4-glyph token), `hook` (pre-commit installer); mints tokens via UUIDv5 over a 5,744-glyph four-block Unicode sign alphabet; DB at `docs/doc-pointer-db.json` in the target repo
- `Makefile` — `test` (cargo fmt/build + `bash -n`), `install` → `~/.local/bin` (release binary, not the cargo-run dev wrapper)

## Key Decisions

- Bash for trivial wrappers; Rust (sole dep: `uuid`) where Unicode/UUID/DB handling is needed
- Pointer identity = token; location resolved on demand by `build`, so links never rot
- Comment-context-only declaration parsing, string-literal quote guard, markdown-fence awareness — no AST, no regex dep
- Byte-exact deterministic DB payload makes `--check` a plain string comparison
- `annotate` covers public API only (`pub fn`, `def`/`defmacro`, JS/TS exports); idempotent via marker-in-doc-block suppression
- Legacy bare flags (`--write`, `--check`, `--install-hook`) still dispatch to subcommands

## Ecosystem Fit

Installed via the monorepo chain `make install-utilities` → `utilities/shell` `SUBDIRS`
(shared `mk/subdirs.mk`) → local `make install` → `~/.local/bin`. Does not use
`share/k8-lib` and does not read `.infra-config.yaml` — pure git tooling, decoupled from
k8s/deploy conventions. Token-encoder golden test is a cross-crate pact with
`repo-lock`'s `glyph.rs` duplicate encoder.

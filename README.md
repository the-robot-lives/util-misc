# misc-git-utils

**Repo:** https://github.com/the-robot-lives/util-misc

Misc git helpers used in trl-infra submodule sweeps — gitlink pinning, branch housekeeping, doc pointers.

## What

Five small commands installed to `~/.local/bin`:

| Command | Purpose |
|---------|---------|
| `gcap <msg>` | `git commit -a -m <msg> && git push origin HEAD` |
| `gp` | `git push origin HEAD` |
| `submodule-diff [repo-root]` | Reads `.gitmodules` recursively, enters each submodule with local changes, and streams `git diff --cached`, `git diff`, and `git diff --no-index /dev/null <untracked-file>` per submodule |
| `submodule-pull [repo-root]` | Reads `.gitmodules`, runs `git pull --ff-only` in each submodule; detached-HEAD submodules are skipped with a note (tag name included when the commit matches a tag) |
| `doc-pointers` | Rust binary that mints stable short pointer tokens (4-glyph hieroglyph alphabet) for docs, backed by `docs/doc-pointer-db.json` (UUID v5 namespaced) |

## Why

The monorepo carries ~90 submodules that need periodic ff-only pulls, diff review across many nested repos, and consistent gitlink housekeeping during sweeps. Doing that by hand per submodule is slow and error-prone; these scripts make the sweep a single command.

## Getting Started

Prerequisites: `cargo` (Rust), `git`, `bash`.

```bash
make install    # builds and installs bin/* to ~/.local/bin
make test
```

## How It Works

- The shell wrappers in `bin/` are thin, `set -euo pipefail` scripts — read them directly for exact behavior; they accept an optional repo root (defaults to cwd).
- `doc-pointers` is the only compiled piece (`src/bin/doc-pointers.rs`): it maps document references to deterministic hieroglyph tokens via UUID v5, stored in `docs/doc-pointer-db.json`, so docs can cite targets without committing long URLs/paths.

Full docs: `docs/` (PROJ-ARCH, PROJ-HOWTO, PROJ-LAYOUT).

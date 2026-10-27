# AGENT-README

> Orientation for AI coding agents working in this repo. Human docs: README.md.

## Type

scripts (bash wrappers + Makefile)

## Purpose

Misc git helpers used in trl-infra submodule sweeps — gitlink pinning and branch housekeeping: `gcap`, `gp`, `submodule-diff`, `submodule-pull`.

## Stack

Thin `set -euo pipefail` bash wrappers in `bin/`, installed via Makefile; optional repo-root argument, defaults to cwd.

## Dev mode

```bash
make install    # installs bin/* to ~/.local/bin
make test
```

## Ports / URLs

- none (library/CLI)

## Tests

```bash
make test
```

## Gotchas

- Also installed from the monorepo root via `make install-utilities`; commands land in `~/.local/bin`.
- Submodules sit on `develop`; PRs target `develop`; `main` is release-triggered (merge to main fires CI/CD).

# PROJ-HOWTO.summary.md — misc-git-utils

Companion to [PROJ-HOWTO.md](PROJ-HOWTO.md): task list + one-line outcomes only.

| Guide | Outcome |
|-------|---------|
| Install these commands on your machine | `gcap`, `gp`, `submodule-pull`, `submodule-diff`, `doc-pointers` land in `~/.local/bin` via `make install`. |
| Commit everything and push in one shot | `gcap "<msg>"` stages tracked changes, commits, and pushes `origin HEAD`. |
| Push my current branch | `gp` runs `git push origin HEAD`. |
| Pull every submodule up to date, safely | `submodule-pull` fast-forwards each `.gitmodules` entry, skipping detached HEADs with a note. |
| Review pending changes across nested submodules before committing | `submodule-diff` streams staged/unstaged/untracked diffs recursively with path-prefixed headers. |
| Mint and use a durable cross-document pointer | `doc-pointers uuid5` + `build --write` creates a token anchor that keeps a `deeplink:` Markdown link resolving after code moves. *(see [howto/doc-pointers-basics.md](howto/doc-pointers-basics.md))* |
| Enforce doc-pointer freshness with a pre-commit hook | `doc-pointers hook` installs a `.git/hooks/pre-commit` that runs `make doc-pointers-check`, failing commits on stale pointer links. *(see [howto/doc-pointers-ci-hook.md](howto/doc-pointers-ci-hook.md))* |

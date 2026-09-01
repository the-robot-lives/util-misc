# PROJ-HOWTO.md — misc-git-utils

Task-oriented guides for the things you'll actually do with this package. See
[PROJ-ARCH.md](PROJ-ARCH.md) for *what it is* and [PROJ-LAYOUT.md](PROJ-LAYOUT.md) for
*where things live*.

## How to: install these commands on your machine

**Goal:** get `gcap`, `gp`, `submodule-pull`, `submodule-diff`, and `doc-pointers` onto
your `PATH`.
**Prereqs:** `cargo`/rustc installed (for `doc-pointers`); `~/.local/bin` on your `PATH`.

1. From this directory:
   ```bash
   make install
   ```
2. This builds the release `doc-pointers` binary and copies all `bin/` scripts (except
   the dev-only `doc-pointers` wrapper) into `~/.local/bin`.

**Verify:**
```bash
which gcap gp submodule-pull submodule-diff doc-pointers
doc-pointers help
```
**Gotchas:**
- If run from the monorepo root instead, use `make install-utilities` (repo root
  `Makefile`) which walks every `utilities/shell/*` package including this one.
- `make test` (cargo fmt/build + `bash -n` on every script) is a good pre-install sanity
  check if you've edited anything here.

## How to: commit everything and push in one shot

**Goal:** stage all tracked changes, commit with a message, and push the current branch
— one command instead of three.
**Prereqs:** `gcap` installed; a remote named `origin`; you actually want *all* tracked
changes committed (this is `git commit -a`, not `git add -A` — new untracked files are
not included).

1. ```bash
   gcap "fix: tighten submodule-diff prefix handling"
   ```

**Verify:** `git log -1 --oneline` shows your commit, and it's visible on `origin` for
your branch.
**Gotchas:**
- No message argument prints a usage error and exits 1 — it never commits without one.
- Untracked (new) files are never picked up by `-a`; `git add` them first if needed.
- Push failures (e.g. non-fast-forward) surface as the underlying `git push` error;
  `gcap` does not retry, rebase, or force-push.

## How to: push my current branch

**Goal:** shorthand for `git push origin HEAD` when you don't want the commit step.
**Prereqs:** `gp` installed.

1. ```bash
   gp
   ```

**Verify:** the branch is up to date on `origin` (`git status` shows nothing to push).
**Gotchas:** same non-fast-forward failure mode as `gcap` — it does not force or set
upstream tracking for you.

## How to: pull every submodule up to date, safely

**Goal:** fast-forward every submodule listed in `.gitmodules`, skipping any that are on
a detached HEAD instead of guessing what to merge.
**Prereqs:** `submodule-pull` installed; run from (or point at) a repo with a
`.gitmodules` file.

1. ```bash
   submodule-pull                # uses current repo root
   submodule-pull /path/to/repo  # or target a specific repo
   ```

**Verify:** output lists `==> <path>` per submodule with either `branch: <name>` +
successful `git pull --ff-only`, or a `note:` line explaining why it was skipped.
**Gotchas:**
- Detached-HEAD submodules are **skipped, not pulled** — you'll see `note: detached HEAD
  at tag <tag> (<sha>); skipping pull` or the untagged variant. This is intentional: the
  tool won't guess which branch you meant to fast-forward to.
- A submodule directory that doesn't exist yet (never initialized) prints a note to run
  `git submodule update --init --recursive` rather than doing it for you.
- Only fast-forward pulls are attempted; a diverged branch fails with git's own
  non-fast-forward error rather than merging or rebasing silently.

## How to: review pending changes across nested submodules before committing

**Goal:** see staged, unstaged, and untracked diffs for every submodule with local
changes — recursively, including submodules-of-submodules — with paths prefixed so the
output reads like one unified diff.
**Prereqs:** `submodule-diff` installed.

1. ```bash
   submodule-diff                # from current repo root
   submodule-diff /path/to/repo  # or target a specific repo
   ```

**Verify:** clean submodules produce no output block for themselves; if none anywhere
have changes you'll see `No submodules with local changes found.`
**Gotchas:**
- It walks recursively — a submodule nested inside a submodule is included with a
  prefixed path like `outer/inner`.
- Untracked files are shown via `git diff --no-index /dev/null <file>` with path
  prefixes rewritten to match, so they appear as normal additions in the stream, not a
  separate "new file" notice.

## How to: mint and use a durable cross-document pointer

Create a token that keeps a Markdown link pointing at the right file:line even after the
target code moves or gets renamed.
→ *See [howto/doc-pointers-basics.md](howto/doc-pointers-basics.md)*

## How to: enforce doc-pointer freshness with a pre-commit hook

Wire `doc-pointers build --check` into CI or a git hook so stale pointer links fail the
build instead of silently rotting.
→ *See [howto/doc-pointers-ci-hook.md](howto/doc-pointers-ci-hook.md)*

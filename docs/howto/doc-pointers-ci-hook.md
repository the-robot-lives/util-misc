# How to: enforce doc-pointer freshness with a pre-commit hook

**Goal:** fail a commit (or CI run) if `docs/doc-pointer-db.json` or an expanded
`deeplink:` link would change — i.e. someone moved anchored code without re-running
`build --write`.
**Prereqs:** `doc-pointers` installed; a `Makefile` in the **target repo** (not this
package) that defines a `doc-pointers-check` target — the hook shells out to `make
doc-pointers-check`, it does not call `doc-pointers` directly.

## 1. Add targets to the target repo's Makefile

Working example from `projects/therobotdrafts/Makefile`:

```makefile
DOC_POINTERS := doc-pointers   # or an absolute path if not on PATH
PROJECT_DIR  := $(CURDIR)

doc-pointers:
	@"$(DOC_POINTERS)" build --root "$(PROJECT_DIR)" --write

doc-pointers-check:
	@"$(DOC_POINTERS)" build --root "$(PROJECT_DIR)" --check

install-doc-pointer-hook:
	@"$(DOC_POINTERS)" hook --root "$(PROJECT_DIR)"
```

## 2. Install the hook

```bash
make install-doc-pointer-hook
# or directly:
doc-pointers hook --root /path/to/target-repo
```

This writes `.git/hooks/pre-commit` (marked with a `# therobotdrafts-doc-pointers`
comment) running `make doc-pointers-check` on every commit.

**Verify:**
```bash
cat .git/hooks/pre-commit   # confirm it's installed and executable
git commit --allow-empty -m "test hook"   # should run doc-pointers-check
```

**Gotchas:**
- **The target repo must already have a `doc-pointers-check` make target.** The hook
  itself only ever runs `make doc-pointers-check` — if that target doesn't exist, every
  commit fails with a `make` "no rule" error, not a doc-pointers error. Add the three
  targets above before installing the hook.
- **Won't silently overwrite an existing hook.** If `.git/hooks/pre-commit` already
  exists and doesn't contain the `# therobotdrafts-doc-pointers` marker, `doc-pointers
  hook` refuses with `<path> already exists and is not managed by this script` — merge
  the check into your existing hook manually in that case.
- `build --check` exits 1 if a write *would* change anything (new/stale pointers,
  un-expanded deeplinks) — run `doc-pointers build --write` locally first to fix before
  committing.
- Legacy flag form still works if you're scripting outside `make`:
  `doc-pointers --check` == `doc-pointers build --check`, and
  `doc-pointers --install-hook` == `doc-pointers hook`.

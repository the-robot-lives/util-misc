# PROJ-FAQ.md — misc-git-utils

Anticipated why/when/compared-to-what questions. See [PROJ-HOWTO.md](PROJ-HOWTO.md) for
procedures and [PROJ-ARCH.md](PROJ-ARCH.md) for design rationale.

## Motivation

### Why would I use `gcap` instead of just typing `git commit -a -m ... && git push`?

Because it's the same two commands with less typing and one less place to typo a `&&`.
There's no hidden behavior, no extra flags, no rebase-and-retry logic — `gcap "msg"` is
`git commit -a -m "msg" && git push origin HEAD`, verbatim. The only real value is muscle
memory: one short command instead of two you have to chain correctly every time.

→ *See [PROJ-HOWTO.md](PROJ-HOWTO.md#how-to-commit-everything-and-push-in-one-shot) to
use it.*

### Why does `doc-pointers` mint tokens instead of just using file:line links directly?

Because a plain `path/to/file.rs:142` link rots the moment someone reformats the file or
renames it, and nobody remembers to grep-and-fix every stale doc link when that happens.
A token (`⟦glyphs⟧`) is a permanent identity; `doc-pointers build` re-resolves its current
`path:line` on every run, so the link self-heals across renames and refactors. The
trade-off: you now depend on a build/check step to keep links honest, and a forgotten
`--write` after moving code leaves the *token* correct but the previously-expanded
Markdown text stale until the next build.

→ *See [howto/doc-pointers-basics.md](howto/doc-pointers-basics.md).*

### Why is `doc-pointers` a Rust binary when every other command here is a bash script?

Because it needs things bash doesn't do well: deterministic UUIDv5 derivation and
non-ASCII (Hieroglyphs-block) token generation with collision checks against a JSON
database. The trivial commands (`gcap`, `gp`, submodule helpers) stay dependency-free
bash since they're one-liners around git plumbing — adding a compiled dependency for
those would be pure overhead. See [PROJ-ARCH.md](PROJ-ARCH.md#key-decisions) for the full
rationale.

### Why does the pre-commit hook run `make doc-pointers-check` instead of calling `doc-pointers build --check` directly?

Because the installed hook body is a fixed, un-parameterized script
(`cd <root> && make doc-pointers-check`) — it has no way to know your binary's path,
your `--db` location, or any extra flags you need, so it delegates all of that to the
target repo's own Makefile target. The upside: you can point at a non-`PATH` binary,
override `--root`/`--db`, or add flags, without `doc-pointers` ever needing its own
config-file format. The cost: the hook is useless until you've defined
`doc-pointers-check` yourself — see the gotcha in
[howto/doc-pointers-ci-hook.md](howto/doc-pointers-ci-hook.md).

## Fit

### When should I *not* use `gcap`?

Whenever you don't actually want *every* tracked change committed. `gcap` runs
`git commit -a`, not `git add -A` — it stages all modified/deleted tracked files but
never picks up new untracked files, and it gives you no chance to review a diff or split
it into multiple commits first. If you're mid-feature with a mix of finished and
half-done tracked changes, or you want a curated commit, use `git add -p` / plain `git
commit` instead.

### When is `submodule-pull` the wrong tool?

When a submodule is on a detached HEAD (e.g. checked out at a tag) — the tool
deliberately **skips** it rather than guessing which branch to fast-forward to, and it
also refuses to do anything but a fast-forward pull, so a diverged branch just fails with
git's normal non-fast-forward error. If you need to rebase, merge, or move a detached
submodule onto a branch, that's a manual `git -C <submodule>` step, not this tool.

→ *See [PROJ-HOWTO.md](PROJ-HOWTO.md#how-to-pull-every-submodule-up-to-date-safely).*

### Is `doc-pointers` useful outside this monorepo?

Yes — it operates entirely against whatever repo you point it at (or the current repo by
default) and stores its database (`docs/doc-pointer-db.json`) in *that* target repo, not
here. Nothing about it assumes Noizu Infra conventions; it's plain git/Markdown tooling.
See [PROJ-ARCH.md](PROJ-ARCH.md#ecosystem-fit).

## Comparison

### How is `submodule-diff` different from plain `git diff` on a repo with submodules?

Plain `git diff` at the superproject level shows submodule pointer bumps, not the actual
changes inside them. `submodule-diff` recurses into every submodule (including
submodules-of-submodules) and streams their staged, unstaged, and untracked diffs as one
unified output with path-prefixed headers — so you can review real content changes across
a whole submodule tree before committing, not just "this submodule moved."

→ *See
[PROJ-HOWTO.md](PROJ-HOWTO.md#how-to-review-pending-changes-across-nested-submodules-before-committing).*

### How does installing via this package differ from `make install-utilities` at the repo root?

Same end state, different scope. Running `make install` here builds and installs only
this package's commands (`gcap`, `gp`, `submodule-pull`, `submodule-diff`,
`doc-pointers`). The repo-root `make install-utilities` walks every `utilities/shell/*`
package — including this one — via the shared `SUBDIRS` mechanism, so it installs
everything at once. Use the local target when iterating on this package alone; use the
root target for a fresh-machine bootstrap.

→ *See [PROJ-HOWTO.md](PROJ-HOWTO.md#how-to-install-these-commands-on-your-machine).*

## Capability

### Can `doc-pointers` tell me a link is stale before I commit, automatically?

Yes — `doc-pointers hook` installs a `pre-commit` hook that runs `build --check`, which
exits 1 if any expanded `deeplink:` link no longer matches the token's current
`path:line`. Without installing the hook, staleness is only caught the next time someone
manually runs `build --check` (e.g. in CI).

→ *See [howto/doc-pointers-ci-hook.md](howto/doc-pointers-ci-hook.md).*

### Can `gcap` retry or force-push if the push is rejected?

No. A non-fast-forward push failure surfaces as git's own error and `gcap` stops there —
it never rebases, force-pushes, or retries on your behalf. This is deliberate: silently
forcing a push is exactly the kind of "convenience" that loses someone else's commits.

## Caveats

### Why can't I put a doc-pointer declaration inside a `.rs`, `.sh`, `.js`, or `.py` file?

Because the scanned-extension list is a fixed, hardcoded allow-list
(`asmdef, cs, css, html, json, md, meta, shader, txt, uxml, yaml, yml`) — not a
configurable option and not a "looks like source" heuristic. The list (plus the
Unity-flavored skip-dirs like `Library`/`DerivedData`/`UserSettings`) reflects the kind of
project `doc-pointers` was first built against, and it's never been extended to common
scripting-language extensions. A declaration inside `.rs`/`.sh`/`.js`/`.py` is silently
invisible to `build`; put it in an adjacent `.md` or `.yaml` file instead if you need to
anchor near source code in one of those languages.

→ *See [howto/doc-pointers-basics.md](howto/doc-pointers-basics.md).*

### Why does the installed pre-commit hook say `# therobotdrafts-doc-pointers` even in a repo unrelated to therobotdrafts?

Because that marker string is a literal hardcoded constant in `doc-pointers`' source, not
something derived from your repo name or config — it exists purely so the tool can detect
"did I already install this hook" (idempotent re-install) and tell its own hook apart from
one it should refuse to overwrite. It has no functional tie to the `therobotdrafts`
project and isn't customizable; don't read anything into the name beyond that it's the
sentinel this specific tool looks for.

### What happens if I run `gcap` with no message?

It prints a usage error and exits 1 without committing anything — there is no default
message and no accidental empty commit.

### Does `submodule-pull` ever touch a submodule that hasn't been initialized yet?

No. If a submodule directory doesn't exist (never `git submodule update --init`'d), it
prints a note telling you to run that command yourself rather than doing it for you —
consistent with the tool's general stance of skipping-with-a-note over guessing.

### Is the doc-pointer database (`docs/doc-pointer-db.json`) safe to hand-edit?

Don't — it's a derived artifact keyed by deterministic UUIDv5s tied to the token glyphs
in source; hand-editing it can desync it from the actual tokens in your files, and the
next `build --write` may not detect or repair that desync since it isn't a full
from-scratch rebuild of every mapping. Treat it as generated output: regenerate via
`doc-pointers build --write`, don't patch it directly.

## Trust

### Does any of this touch secrets, credentials, or `.infisical`-managed data?

No. Unlike most sibling utilities under `utilities/shell/`, this package does not source
`share/k8-lib` and does not read `.infra-config.yaml` — it's pure git tooling with zero
coupling to the k8s/deploy/secrets conventions used elsewhere in the monorepo. See
[PROJ-ARCH.md](PROJ-ARCH.md#ecosystem-fit).

### Where does `doc-pointers`' state live — here, or in my repo?

In *your* target repo, always. This package ships no data of its own; running
`doc-pointers build` against a repo writes `docs/doc-pointer-db.json` inside that repo,
not inside `misc-git-utils`. Uninstalling the tool leaves that file behind (harmless,
just inert JSON) since it belongs to the target repo, not this package.

# PROJ-FAQ.summary.md — misc-git-utils

Question index only. Full answers: [PROJ-FAQ.md](PROJ-FAQ.md).

## Motivation
- Why would I use `gcap` instead of just typing `git commit -a -m ... && git push`?
- Why does `doc-pointers` mint tokens instead of just using file:line links directly?
- Why is `doc-pointers` a Rust binary when every other command here is a bash script?
- Why does the pre-commit hook run `make doc-pointers-check` instead of calling `doc-pointers build --check` directly?

## Fit
- When should I *not* use `gcap`?
- When is `submodule-pull` the wrong tool?
- Is `doc-pointers` useful outside this monorepo?

## Comparison
- How is `submodule-diff` different from plain `git diff` on a repo with submodules?
- How does installing via this package differ from `make install-utilities` at the repo root?

## Capability
- Can `doc-pointers` tell me a link is stale before I commit, automatically?
- Can `gcap` retry or force-push if the push is rejected?

## Caveats
- Why can't I put a doc-pointer declaration inside a `.rs`, `.sh`, `.js`, or `.py` file?
- Why does the installed pre-commit hook say `# therobotdrafts-doc-pointers` even in a repo unrelated to therobotdrafts?
- What happens if I run `gcap` with no message?
- Does `submodule-pull` ever touch a submodule that hasn't been initialized yet?
- Is the doc-pointer database (`docs/doc-pointer-db.json`) safe to hand-edit?

## Trust
- Does any of this touch secrets, credentials, or `.infisical`-managed data?
- Where does `doc-pointers`' state live — here, or in my repo?

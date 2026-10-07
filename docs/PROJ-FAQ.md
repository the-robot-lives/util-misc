# FAQ — misc-git-utils

## Does `gcap` stage untracked files?

No. It runs `git commit -a -m`, which includes modified and deleted tracked files. Stage new files explicitly.

## Does `gcap` retry or force a rejected push?

No. It stops at Git's error.

## Does `submodule-pull` switch detached HEADs to a branch?

No. Detached checkouts are skipped. Pulls on branches use `--ff-only`.

## What does `submodule-diff` add over a root `git diff`?

It enters nested submodules and prints staged, unstaged, and untracked file changes.

## Where is `doc-pointers`?

The CLI and MCP service live in [doc-pointers](https://github.com/the-robot-lives/doc-pointers). Install it from that repository.

# Project Architecture — misc-git-utils
<!-- 〚📁:38c869ec-88d8-542b-8e8e-6eb3a56897ad〛 Project Architecture :: Four standalone Bash commands, no shared runtime. -->


## Overview

This package installs four standalone Bash commands for Git and submodule housekeeping. Each command is a self-contained script in `bin/`; there is no shared runtime or persistent state.

```mermaid
flowchart LR
  A[make install] --> B[bin/gcap and bin/gp]
  A --> C[bin/submodule-pull and bin/submodule-diff]
  B --> D[~/.local/bin]
  C --> D
```

| Command | Behavior |
|---|---|
| `gcap` | Commit all tracked changes with the supplied message, then push `origin HEAD`. |
| `gp` | Push `origin HEAD`. |
| `submodule-pull` | Fast-forward initialized submodules; skip detached HEADs. |
| `submodule-diff` | Show staged, unstaged, and untracked changes across nested submodules. |

→ *Components ↔ directories: see [PROJ-LAYOUT.md](PROJ-LAYOUT.md)*

The doc-pointer CLI and MCP service are maintained together in [doc-pointers](https://github.com/the-robot-lives/doc-pointers). This package no longer builds or installs that CLI.

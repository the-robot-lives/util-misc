# Architecture Summary — misc-git-utils

Four independent Bash commands in `bin/` are installed to `~/.local/bin` by `make install`: `gcap`, `gp`, `submodule-pull`, and `submodule-diff`. There is no shared runtime or persistent state. The doc-pointer CLI and MCP service live in [doc-pointers](https://github.com/the-robot-lives/doc-pointers).

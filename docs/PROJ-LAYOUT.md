# Project Layout — misc-git-utils

```text
misc-git-utils/
├── bin/
│   ├── gcap
│   ├── gp
│   ├── submodule-diff
│   └── submodule-pull
├── docs/
├── Makefile
└── README.md
```

`make test` checks Bash syntax for every script. `make install` copies the four scripts into `~/.local/bin`. The doc-pointer CLI and MCP service live in [doc-pointers](https://github.com/the-robot-lives/doc-pointers).

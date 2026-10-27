# Project Layout — misc-git-utils
<!-- 〚📁:06ae3269-0033-598e-90ec-aa8d081062c6〛 Project Layout :: Full repo tree with per-file descriptions. -->


```text
misc-git-utils/
├── bin/                        # The four commands — self-contained Bash scripts
│   ├── gcap                    #   git commit -a -m <msg> && git push origin HEAD
│   ├── gp                      #   git push origin HEAD
│   ├── submodule-diff          #   stream staged/unstaged/untracked diffs across nested submodules
│   └── submodule-pull          #   ff-only pull each initialized submodule; skips detached HEADs
├── docs/                       # Self-documentation (each PROJ-* has a .summary.md twin)
│   ├── PROJ-ARCH.md            #   Architecture overview
│   ├── PROJ-FAQ.md             #   Command gotchas
│   ├── PROJ-HOWTO.md           #   Usage walkthrough
│   ├── PROJ-LAYOUT.md          #   This file
│   ├── PROJ-SCHEMA.md          #   Data/schema statement
│   └── THREAT-MODEL.md         #   Trust boundaries + vulnerability register
├── .gitignore                  # Editor/OS artifacts + canonical .claude/worktrees/ placement
├── AGENT.md                    # Agent instructions (Codex/Grok restyle of CLAUDE.md)
├── AGENTS.md                   # Same content as AGENT.md (agent-harness convention)
├── CHANGELOG.md                # Dated work log
├── CLAUDE.md                   # Agent instructions: branch policy, worktree convention
├── Makefile                    # compile/test/install/clean — installs bin/* to ~/.local/bin
├── README.md                   # Project entry point
└── merge-notes.md              # Historical note: sep-1 branch sweep, develop-vs-main state
```

`make test` runs `bash -n` over every script in `bin/`. `make install` copies them into `~/.local/bin` (mode 755). There are no build outputs, dependencies, or generated files.

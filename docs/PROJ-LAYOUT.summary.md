# Project Layout Summary — misc-git-utils

```
misc-git-utils/
├── bin/                        # git helper scripts (bash)
│   ├── gcap                    #   commit-all + push
│   ├── gp                      #   push HEAD
│   ├── submodule-pull          #   ff-only pull all submodules
│   ├── submodule-diff          #   recursive submodule diffs
│   └── doc-pointers            #   cargo-run dev wrapper
├── src/bin/doc-pointers.rs     # Rust doc-pointer scanner/DB tool (build/annotate/uuid5/hook)
├── docs/                       # PROJ-* docs + summaries + howto/ topic guides
│   └── howto/                  #   doc-pointers basics / annotate / CI hook
├── .gitignore                  # target/, env files, swap files
├── CLAUDE.md                   # Claude Code guidance
├── Cargo.toml                  # Rust package (dep: uuid)
├── Cargo.lock                  # locked deps
├── Makefile                    # test / install → ~/.local/bin
├── CHANGELOG.md                # change log
└── merge-notes.md              # annotate-merge working notes
```

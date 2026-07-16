# Project Layout Summary — misc-git-utils

```
misc-git-utils/
├── bin/                        # git helper scripts (bash)
│   ├── gcap                    #   commit-all + push
│   ├── gp                      #   push HEAD
│   ├── submodule-pull          #   ff-only pull all submodules
│   ├── submodule-diff          #   recursive submodule diffs
│   └── doc-pointers            #   cargo-run dev wrapper
├── src/bin/doc-pointers.rs     # Rust doc-pointer scanner/DB tool
├── docs/                       # PROJ-LAYOUT.md + this summary
├── .gitignore                  # target/, env files, swap files
├── Cargo.toml                  # Rust package (dep: uuid)
├── Cargo.lock                  # locked deps
└── Makefile                    # test / install → ~/.local/bin
```

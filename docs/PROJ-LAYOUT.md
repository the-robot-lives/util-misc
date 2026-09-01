# Project Layout — misc-git-utils

Terminal utility package: miscellaneous git helper commands installed to `~/.local/bin` via `make install`. Mixed bash (`bin/`) + Rust (`src/bin/doc-pointers.rs`) toolset.

```
misc-git-utils/
├── bin/                        # Executable git utilities (bash, installed as-is)
│   ├── gcap                    #   git commit -a -m "<msg>" && push origin HEAD
│   ├── gp                      #   git push origin HEAD
│   ├── submodule-pull          #   ff-only pull each .gitmodules submodule; skips detached HEADs (notes matching tag)
│   ├── submodule-diff          #   recursive submodule diff: staged, unstaged, and untracked files
│   └── doc-pointers            #   dev wrapper — `cargo run --bin doc-pointers` (release binary installed instead)
├── src/                        # Rust sources
│   └── bin/
│       └── doc-pointers.rs     #   doc-pointer scanner/DB tool: build/annotate/uuid5/hook subcommands, 4-glyph Unicode tokens, scan filter, check/write modes, pre-commit hook install; DB at docs/doc-pointer-db.json (in the target repo)
├── docs/                       # Documentation
│   ├── PROJ-LAYOUT.md          #   This file
│   ├── PROJ-LAYOUT.summary.md  #   Tree-only companion (keep in sync)
│   ├── PROJ-ARCH.md            #   Architecture overview (system diagram, design decisions)
│   ├── PROJ-ARCH.summary.md    #   Architecture quick reference
│   ├── PROJ-SCHEMA.md          #   Config/data artifacts: doc-pointer DB JSON, hook, CLI grammar
│   ├── PROJ-HOWTO.md           #   Usage how-to guide
│   ├── PROJ-HOWTO.summary.md   #   How-to quick reference
│   ├── PROJ-FAQ.md             #   FAQ / troubleshooting
│   ├── PROJ-FAQ.summary.md     #   FAQ quick reference
│   └── howto/                  #   doc-pointers topic guides
│       ├── doc-pointers-basics.md
│       ├── doc-pointers-annotate.md
│       └── doc-pointers-ci-hook.md
├── target/                     # Cargo build output (gitignored — not documented)
├── .gitignore                  # Ignores target/, .env, .envrc.local, editor swap files
├── Cargo.toml                  # Rust package `misc-git-utils` (edition 2021; dep: uuid v4/v5)
├── Cargo.lock                  # Locked Rust dependency versions (committed)
├── Makefile                    # test (cargo fmt/build + bash -n bin/*), install → ~/.local/bin
├── CHANGELOG.md                # Release/change log
└── merge-notes.md              # Working notes from the doc-pointers annotate merge
```

## Key Files Requiring Setup

| File | Action |
|------|--------|
| — | No manual configuration required; run `make install` to build `doc-pointers` (release) and copy all `bin/` scripts to `~/.local/bin`. |

## Notes

- `make install` installs the compiled release `doc-pointers` binary, not the `bin/doc-pointers` cargo-run wrapper; all other `bin/` scripts are installed verbatim.
- `make test` runs `cargo fmt --check`, a quiet cargo build, and `bash -n` syntax checks on every `bin/` script.
- `doc-pointers` maintains its pointer database at `docs/doc-pointer-db.json` in whatever repo it is run against (default path; not a file of this project — this repo itself does not carry a pointer DB).

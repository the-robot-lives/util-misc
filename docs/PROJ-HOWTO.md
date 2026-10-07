# How to use misc-git-utils

Install the four shell commands:

```bash
make test
make install
```

`make install` copies `gcap`, `gp`, `submodule-pull`, and `submodule-diff` to `~/.local/bin`.

## Commit and push tracked changes

```bash
gcap "commit message"
```

`gcap` runs `git commit -a -m` and then `git push origin HEAD`. New untracked files must be staged separately.

## Push the current branch

```bash
gp
```

## Update initialized submodules

```bash
submodule-pull /path/to/repo
```

The command pulls with `--ff-only` and skips detached HEADs.

## Review nested submodule changes

```bash
submodule-diff /path/to/repo
```

The command includes staged, unstaged, and untracked changes.

For doc-pointer CLI use and installation, see [doc-pointers](https://github.com/the-robot-lives/doc-pointers).

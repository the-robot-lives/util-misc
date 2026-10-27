# Threat Model Summary — misc-git-utils

Four Bash wrappers around local `git`; no service, listener, or persistence. Boundaries: operator shell → script → local repos and `.gitmodules` submodule paths → configured remotes. Register: T-001 (Medium — `submodule-diff` streams untracked non-gitignored file contents), T-002 (`.gitmodules` paths trusted verbatim), T-003 (fast-forward pulls adopt remote state), T-004 (`gcap` bulk commit+push without review). 1 partial · 3 open — accepted residual risk for a single-operator utility.

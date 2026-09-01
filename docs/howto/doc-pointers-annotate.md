# doc-pointers annotate — mass-marking public functions

`doc-pointers annotate` walks a tree and inserts a `⟦code⟧ Name :: Description` marker
above every **public/exported** function that doesn't already carry one in its
doc/comment block, then records the database and expands deeplinks in the same run.

## What gets annotated

| Language | Matched | Not matched |
|---|---|---|
| Rust (`.rs`) | `pub fn` (incl. `pub async/unsafe/const/extern "C" fn`) | `pub(crate)`/`pub(super)` (internal API by declaration), private `fn`, trait methods without `pub` |
| Elixir (`.ex`; `.exs` only with `--lang exs`) | `def`, `defmacro` — first clause per function name | `defp`, `defmacrop`, `defmodule`, `defdelegate`, `defimpl`, `def unquote(...)` heads |
| JS/TS (`.js .ts .mjs .tsx`) | `export [default] [async] function NAME`, function-shaped `export const NAME = …`, `[module.]exports.NAME =` | `module.exports = { … }` object exports (flagged for manual review), non-function `export const`, `.d.ts`, `*.min.js` |

Detection is line-anchored at indentation — string literals mentioning `pub fn` etc. on
mid-line positions never match. Lines longer than 500 chars are skipped (minification
tripwire). Files under `deps/`, `_build/`, `node_modules/`, `.next/`, `.claude/`, `dist/`,
`target/` etc. are never scanned.

## Usage

```sh
# Dry-run (default): report what would be inserted, nothing changes
doc-pointers annotate --root . --include utilities --include libs

# Apply: insert markers, write docs/doc-pointer-db.json, expand deeplinks
doc-pointers annotate --root . --include utilities --include libs --write
```

Scoping flags (`--include`/`--exclude`, root-relative prefixes, repeatable) are shared
with `build`. In a monorepo always scope the root invocation — never sweep subtrees that
own their own database (e.g. `projects/therobotdrafts`).

## Determinism & collisions

Codes are minted as `uuid5("doc-pointers:<relpath>::<fn-name>")` — the same tree yields
the same codes on any machine. Collisions are checked against the database **and every
marker already present in the scanned tree** (so branch-minted markers count), plus codes
minted earlier in the same run; on collision the seed is retried with `:1`, `:2`, …
The 4-glyph space is 1072⁴ ≈ 1.32×10¹² (~2^40); at 7,000 pointers the expected number of
birthday collisions is ≈ 2×10⁻⁵ — the retry loop is overwhelming margin.

## Idempotency

A declaration is skipped when the contiguous comment/attribute/doc block immediately
above it already contains `⟦…⟧` — re-running annotate is a no-op, and a human may move
the marker anywhere within that block without it being re-added.

## Descriptions

The first sentence of the existing doc block (`///`, `@doc`, JSDoc) is used when present
(truncated to ~100 chars, embedded `::` softened to `:`); otherwise a placeholder
`auto-generated pointer for public function NAME` is inserted. Enrich placeholders
opportunistically when touching the code — the marker line is the natural place.

## Merge conflicts on the database

The DB is derived, deterministic, and regenerable. On conflict: take either side and run
`doc-pointers build --write` (or `make doc-pointers`) to regenerate.

# How to: mint and use a durable cross-document pointer

**Goal:** place a stable anchor in source/docs that a Markdown `deeplink:` reference can
target forever, even after the anchored code moves, renames, or shifts line numbers.
**Prereqs:** `doc-pointers` installed (`make install` in this package); run from inside
the target repo (the one whose files you want to anchor/link).

## 1. Mint a token

```bash
doc-pointers uuid5 "routing table init"
```

This prints a deterministic UUIDv5, its 4-glyph code, the ready-to-paste marker, and
copies a `⟦token⟧ Name :: Description` declaration to your clipboard (see Gotchas below —
this step needs `pbcopy`).

## 2. Place the declaration at the anchor

Paste it into a comment at the line you want addressable, e.g. in a `.md` or `.yaml`
file:

```
# ⟦𓆴𓎲𓋝𓁅⟧ routing table init :: builds the initial route table on boot
```

The token must sit in a recognized comment context (`//`, `#`, `<!--`, `/*`, `*`, `--`,
`;`) or alone on its own line — this keeps it from ever colliding with a string literal.

## 3. Reference it from Markdown

```markdown
See [routing init](deeplink:⟦𓆴𓎲𓋝𓁅⟧) for details.
```

## 4. Build the database and expand the link

```bash
doc-pointers build --write
```

This scans the repo, (re)writes `docs/doc-pointer-db.json`, and rewrites every
`deeplink:⟦token⟧` reference into a concrete `path:line?code=⟦token⟧` target.

**Verify:**
```bash
doc-pointers build          # dry run — reports count, no changes
git diff -- '*.md' docs/doc-pointer-db.json
```
Confirm the deeplink now resolves to the real file and line, and the JSON DB contains
your token.

**Gotchas:**
- **Clipboard step needs `pbcopy`.** `doc-pointers uuid5` unconditionally shells out to
  `pbcopy` unless you pass `--no-clipboard`; on Linux (no `pbcopy` on `PATH`) the command
  errors out after printing the token. Use:
  ```bash
  doc-pointers uuid5 "routing table init" --no-clipboard
  ```
  and copy the `clipboard:` line it prints manually.
- **Only specific file extensions are scanned**: `asmdef, cs, css, ex, exs, html, js,
  json, md, meta, mjs, rs, shader, ts, tsx, txt, uxml, yaml, yml`. A declaration inside a
  `.sh` or `.py` file is still silently invisible to `build` — put those declarations in
  adjacent `.md`/`.yaml` docs. (`.rs`/`.ex`/`.js`/`.ts` source anchors register as of the
  `annotate` release; see `doc-pointers-annotate.md` for mass-annotating public functions.)
- `build` is **read-only by default**. Without `--write` it only reports how many
  declarations it found and any duplicate-token errors — nothing is written until you add
  `--write`.
- Duplicate tokens across files are reported as errors but don't stop the scan; re-mint a
  fresh token with `doc-pointers uuid5` rather than reusing one by hand.

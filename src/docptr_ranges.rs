//! Scope-prefixed doc pointers, logic blocks, naive ranges, layered index, and impact analysis.
//!
//! Prefix grammar (declaration, in a comment context like the legacy `⟦CODE⟧` form):
//!   emoji canonical:  📁⟦CODE⟧ Name :: Desc   🧩⟦CODE⟧ …   🔧⟦CODE⟧ …   🔀⟦CODE⟧ …
//!   ascii read-compat: @f:⟦CODE⟧ …  @m:⟦CODE⟧ …  @fn:⟦CODE⟧ …  @l:⟦CODE⟧ …
//! Logic blocks:
//!   [[🔀:CODE]] Label :: Desc   …   [[/🔀:CODE]]
//! Open/close tokens must match; blocks nest only when fully contained; open and close
//! may live in different files; unterminated blocks are errors under `scan --strict`.
//! Ranges are advisory naive per-language boundaries; checksum refresh covers drift.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::{rel_path, valid_code, DOC_POINTER_NAMESPACE};

// ---------------------------------------------------------------------------
// Kinds and prefixes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    File,
    Module,
    Function,
    Logic,
}

impl Kind {
    pub fn glyph(self) -> &'static str {
        match self {
            Kind::File => "📁",
            Kind::Module => "🧩",
            Kind::Function => "🔧",
            Kind::Logic => "🔀",
        }
    }
    pub fn ascii(self) -> &'static str {
        match self {
            Kind::File => "@f:",
            Kind::Module => "@m:",
            Kind::Function => "@fn:",
            Kind::Logic => "@l:",
        }
    }
    fn from_glyph(g: &str) -> Option<Kind> {
        Some(match g {
            "📁" => Kind::File,
            "🧩" => Kind::Module,
            "🔧" => Kind::Function,
            "🔀" => Kind::Logic,
            _ => return None,
        })
    }
    fn from_ascii(a: &str) -> Option<Kind> {
        Some(match a {
            "@f:" => Kind::File,
            "@m:" => Kind::Module,
            "@fn:" => Kind::Function,
            "@l:" => Kind::Logic,
            _ => return None,
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Kind::File => "file",
            Kind::Module => "module",
            Kind::Function => "function",
            Kind::Logic => "logic",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Decl {
    pub kind: Kind,
    pub code: String,
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub file: String,
    /// 1-based inclusive; logic blocks use the open marker line unless closed elsewhere.
    pub line_start: usize,
    pub line_end: usize,
    /// Exact open marker line for logic blocks (start file).
    pub close_file: Option<String>,
    pub close_line: Option<usize>,
}

/// Parse one line for a scope-prefixed declaration (emoji or ascii form).
/// Returns the kind, code, name, description.
pub fn parse_prefixed(line: &str) -> Option<(Kind, String, String, String)> {
    let (kind, after) = find_prefix(line)?;
    let start = after.find('⟦')?;
    let after_start = start + '⟦'.len_utf8();
    let end_offset = after[after_start..].find('⟧')?;
    let end = after_start + end_offset;
    let code = &after[after_start..end];
    if code.chars().count() != 4 || !valid_code(code) {
        return None;
    }
    let rest = after[end + '⟧'.len_utf8()..].trim_start();
    let (name, description) = rest.split_once("::")?;
    Some((
        kind,
        code.to_string(),
        name.trim().to_string(),
        description.trim().to_string(),
    ))
}

fn find_prefix(line: &str) -> Option<(Kind, &str)> {
    // Emoji form may be preceded by comment text; ascii form too.
    if let Some(pos) = line.find("⟦") {
        let prefix = &line[..pos];
        for glyph in ["📁", "🧩", "🔧", "🔀"] {
            if let Some(gpos) = prefix.rfind(glyph) {
                if prefix[gpos + glyph.len()..].trim().is_empty() {
                    return Some((Kind::from_glyph(glyph).unwrap(), &line[pos..]));
                }
            }
        }
    }
    for ascii in ["@f:", "@m:", "@fn:", "@l:"] {
        if let Some(pos) = line.find(ascii) {
            let after = &line[pos + ascii.len()..];
            if after.trim_start().starts_with("⟦") {
                return Some((Kind::from_ascii(ascii).unwrap(), after.trim_start()));
            }
        }
    }
    None
}

/// Parse a logic-block open line: `[[🔀:CODE]] Label :: Desc`
pub fn parse_logic_open(line: &str) -> Option<(String, String, String)> {
    let pos = line.find("[[🔀:")?;
    let rest = &line[pos + "[[🔀:".len()..];
    let close = rest.find(']')?;
    let code = &rest[..close];
    if !rest[close..].starts_with("]]") {
        return None;
    }
    if code.chars().count() != 4 || !valid_code(code) {
        return None;
    }
    let rest = rest[close + 2..].trim();
    let (name, description) = rest.split_once("::")?;
    Some((
        code.to_string(),
        name.trim().to_string(),
        description.trim().to_string(),
    ))
}

/// Parse a logic-block close line: `[[/🔀:CODE]]`
pub fn parse_logic_close(line: &str) -> Option<String> {
    let pos = line.find("[[/🔀:")?;
    let rest = &line[pos + "[[/🔀:".len()..];
    let close = rest.find(']')?;
    if !rest[close..].starts_with("]]") {
        return None;
    }
    let code = &rest[..close];
    if code.chars().count() != 4 || !valid_code(code) {
        return None;
    }
    Some(code.to_string())
}

// ---------------------------------------------------------------------------
// Language table (naive, advisory boundaries)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Elixir,
    Rust,
    Go,
    Js,
    Python,
    C,
}

pub fn lang_for_path(path: &str) -> Option<Lang> {
    let ext = path.rsplit('.').next()?;
    Some(match ext {
        "ex" | "exs" => Lang::Elixir,
        "rs" => Lang::Rust,
        "go" => Lang::Go,
        "js" | "ts" | "mjs" | "jsx" | "tsx" => Lang::Js,
        "py" => Lang::Python,
        "c" | "h" => Lang::C,
        _ => return None,
    })
}

pub fn lang_name(lang: Lang) -> &'static str {
    match lang {
        Lang::Elixir => "elixir",
        Lang::Rust => "rust",
        Lang::Go => "go",
        Lang::Js => "js",
        Lang::Python => "python",
        Lang::C => "c",
    }
}

fn opens_block(lang: Lang, trimmed: &str) -> bool {
    match lang {
        Lang::Elixir => trimmed.starts_with("defmodule ")
            || trimmed.starts_with("def ")
            || trimmed.starts_with("defp ")
            || trimmed.starts_with("defmacro ")
            || trimmed.starts_with("if ")
            || trimmed.starts_with("unless ")
            || trimmed.starts_with("case ")
            || trimmed.starts_with("cond")
            || trimmed.starts_with("with ")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("fn "),
        Lang::Rust | Lang::Go | Lang::Js | Lang::C => trimmed.contains('{'),
        Lang::Python => trimmed.starts_with("def ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("if ")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("with ")
            || trimmed.starts_with("try"),
    }
}

fn closes_block(lang: Lang, trimmed: &str) -> bool {
    match lang {
        Lang::Elixir => trimmed == "end",
        Lang::Rust | Lang::Go | Lang::Js | Lang::C => trimmed.contains('}'),
        Lang::Python => false, // dedent handled separately
    }
}

/// Advisory end line (1-based inclusive) for a construct opening at `start_line`
/// (1-based) whose declaration line is `lines[start_line-1]`.
pub fn naive_range(lang: Lang, lines: &[&str], start_line: usize) -> usize {
    match lang {
        Lang::Python => {
            let indent_of = |l: &str| l.chars().take_while(|c| c.is_whitespace()).count();
            let base = indent_of(lines[start_line - 1]);
            let mut end = start_line;
            for (idx, line) in lines.iter().enumerate().skip(start_line) {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if indent_of(line) <= base {
                    break;
                }
                end = idx + 1;
            }
            // include trailing contiguous blank/comment lines before dedent
            let mut last = end;
            for line in lines.iter().skip(end) {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    last += 1;
                } else {
                    break;
                }
            }
            last
        }
        _ => {
            let mut depth: i64 = 0;
            for (idx, line) in lines.iter().enumerate().skip(start_line - 1) {
                let trimmed = line.trim();
                if lang == Lang::Elixir {
                    if opens_block(lang, trimmed) && !trimmed.contains("do:") {
                        depth += 1;
                    }
                    // `def x, do: ...` is single-line; starts_with("def ") without "do" body
                    if trimmed.ends_with(", do: something") {
                        // crude; treat single-line do: as non-nesting
                    }
                    if closes_block(lang, trimmed) {
                        depth -= 1;
                        if depth <= 0 {
                            return idx + 1;
                        }
                    }
                } else {
                    depth += trimmed.matches('{').count() as i64;
                    depth -= trimmed.matches('}').count() as i64;
                    if depth <= 0 && idx + 1 >= start_line {
                        return idx + 1;
                    }
                }
            }
            lines.len()
        }
    }
}

/// Full-file scan producing declarations with ranges. Logic blocks may close in
/// another file; unterminated closes are ignored here and reported by `strict_errors`.
pub fn scan_file(
    root: &Path,
    file: &Path,
    text: &str,
    uuid_of: &dyn Fn(&str) -> String,
) -> (Vec<Decl>, Vec<String>) {
    let mut decls = Vec::new();
    let mut errors = Vec::new();
    let relfile = rel_path(file, root);
    let lines: Vec<&str> = text.lines().collect();
    let lang = lang_for_path(&relfile);

    // Legacy unprefixed declarations are treated as kind-less file-level anchors:
    // skip here — `build` handles them.

    // Logic blocks: track open blocks across the file; close may come later in file
    // or another file entirely (cross-file handled by the caller's index pass).
    let mut open_blocks: Vec<(String, usize)> = Vec::new(); // (code, line)
    for (idx, line) in lines.iter().enumerate() {
        if let Some((code, name, description)) = parse_logic_open(line) {
            if open_blocks.iter().any(|(c, _)| c == &code) {
                errors.push(format!(
                    "{relfile}:{}: duplicate open for logic block {code}",
                    idx + 1
                ));
                continue;
            }
            open_blocks.push((code.clone(), idx + 1));
            let seed = format!("{relfile}::#{name}");
            let uuid = uuid_of(&seed);
            decls.push(Decl {
                kind: Kind::Logic,
                code,
                uuid,
                name,
                description,
                file: relfile.clone(),
                line_start: idx + 1,
                line_end: idx + 1, // finalized below
                close_file: None,
                close_line: None,
            });
            continue;
        }
        if let Some(code) = parse_logic_close(line) {
            match open_blocks.iter().position(|(c, _)| c == &code) {
                Some(pos) => {
                    let (_, open_line) = open_blocks.remove(pos);
                    if let Some(decl) = decls.iter_mut().find(|d| d.code == code) {
                        decl.line_end = idx + 1;
                        decl.close_line = Some(idx + 1);
                        let _ = open_line;
                    }
                }
                None => errors.push(format!(
                    "{relfile}:{}: close for logic block {code} has no open in this file",
                    idx + 1
                )),
            }
        }
    }

    // Prefixed declarations.
    for (idx, line) in lines.iter().enumerate() {
        let Some((kind, code, name, description)) = parse_prefixed(line) else {
            continue;
        };
        if kind == Kind::Logic {
            continue; // handled via blocks above
        }
        let seed = match kind {
            Kind::File => relfile.clone(),
            Kind::Module | Kind::Function => format!("{relfile}::{name}"),
            Kind::Logic => unreachable!(),
        };
        let uuid = uuid_of(&seed);
        let (line_start, line_end) = match kind {
            Kind::File => (1, lines.len()),
            Kind::Module | Kind::Function => match lang {
                Some(l) => (idx + 1, naive_range(l, &lines, idx + 1)),
                None => (idx + 1, idx + 1),
            },
            Kind::Logic => unreachable!(),
        };
        decls.push(Decl {
            kind,
            code,
            uuid,
            name,
            description,
            file: relfile.clone(),
            line_start,
            line_end,
            close_file: None,
            close_line: None,
        });
    }

    // Unterminated opens in this file are cross-file candidates; collect_prefixed
    // decides whether they are errors (strict) or tolerated (lax).

    (decls, errors)
}

// ---------------------------------------------------------------------------
// Unified diff parsing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: String,
    /// (start, count) on the new side, 1-based; count 0 means pure deletion.
    pub hunks: Vec<(usize, usize)>,
}

/// Parse a unified diff into per-file new-side hunk ranges.
pub fn parse_diff(diff: &str) -> Vec<FileDiff> {
    let mut files = Vec::new();
    let mut current: Option<FileDiff> = None;
    let mut in_hunk = false;
    let mut new_line = 0usize;
    let mut new_count = 0usize;
    for line in diff.lines() {
        if line.starts_with("diff --git ") {
            if let Some(prev) = current.take() {
                files.push(prev);
            }
            in_hunk = false;
            continue;
        }
        if let Some(rest) = line.strip_prefix("+++ ") {
            let path = rest
                .strip_prefix("b/")
                .unwrap_or(rest)
                .trim()
                .to_string();
            current = if path != "/dev/null" {
                Some(FileDiff { path, hunks: Vec::new() })
            } else {
                None
            };
            in_hunk = false;
            continue;
        }
        if line.starts_with("@@") {
            let Some(rest) = line.split("@@").nth(1) else {
                continue;
            };
            // rest like " -a,b +c,d"
            let new_part = rest.split_whitespace().find(|p| p.starts_with('+'));
            let Some(new_part) = new_part else { continue };
            let nums = new_part.trim_start_matches('+');
            let (start, count) = match nums.split_once(',') {
                Some((s, c)) => (s.parse::<usize>().unwrap_or(1), c.parse::<usize>().unwrap_or(1)),
                None => (nums.parse::<usize>().unwrap_or(1), 1),
            };
            new_line = start;
            new_count = count;
            if let Some(file) = current.as_mut() {
                file.hunks.push((start, count));
            }
            in_hunk = true;
            continue;
        }
        if in_hunk {
            if let Some(file) = current.as_mut() {
                if line.starts_with('+') || line.starts_with(' ') {
                    if new_count > 0 {
                        new_line += 1;
                        new_count -= 1;
                    }
                }
            }
        }
    }
    if let Some(prev) = current.take() {
        files.push(prev);
    }
    files.retain(|f| !f.hunks.is_empty());
    files
}

// ---------------------------------------------------------------------------
// Impact analysis
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ImpactHit {
    pub code: String,
    pub uuid: String,
    pub kind: &'static str,
    pub name: String,
    pub file: String,
    pub line_start: usize,
    pub line_end: usize,
    pub component: String,
}

/// One regex-free pass per changed file: extract declarations + logic blocks,
/// then report those overlapping any changed hunk. `components` maps file
/// prefixes to component names.
pub fn impact(
    root: &Path,
    diff: &str,
    components: &[(String, String)],
) -> Result<Vec<ImpactHit>, String> {
    let filediffs = parse_diff(diff);
    let mut hits = Vec::new();
    for fd in &filediffs {
        let path = root.join(&fd.path);
        let Ok(text) = fs::read_to_string(&path) else {
            continue; // deleted or unreadable
        };
        let uuid_of = |seed: &str| uuid5_str(seed);
        let (decls, _) = scan_file(root, &path, &text, &uuid_of);
        for decl in decls {
            let overlapped = fd.hunks.iter().any(|&(start, count)| {
                let hunk_end = if count == 0 { start } else { start + count - 1 };
                decl.line_start <= hunk_end && decl.line_end >= start
            });
            if !overlapped {
                continue;
            }
            let component = components
                .iter()
                .find(|(prefix, _)| fd.path.starts_with(prefix.as_str()))
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            hits.push(ImpactHit {
                code: decl.code,
                uuid: decl.uuid,
                kind: decl.kind.name(),
                name: decl.name,
                file: fd.path.clone(),
                line_start: decl.line_start,
                line_end: decl.line_end,
                component,
            });
        }
    }
    Ok(hits)
}

pub fn uuid5_str(seed: &str) -> String {
    // Mirrors generate_uuid5_code without collision attempts: deterministic uuid for
    // the seed, first salt slot.
    let value = uuid::Uuid::new_v5(
        &DOC_POINTER_NAMESPACE,
        format!("doc-pointers:{seed}").as_bytes(),
    );
    value.to_string()
}

/// Load component prefix table from `.trr/components/*/review.yaml`.
/// Expected minimal shape: `paths:\n  - prefix/one\n  - prefix/two`
pub fn load_components(root: &Path) -> Vec<(String, String)> {
    let dir = root.join(".trr/components");
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(&dir) else {
        return out;
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let review = entry.path().join("review.yaml");
        let Ok(text) = fs::read_to_string(&review) else {
            continue;
        };
        let name = entry.file_name().to_string_lossy().to_string();
        let mut in_paths = false;
        for line in text.lines() {
            if line.trim_start().starts_with("paths:") {
                in_paths = true;
                continue;
            }
            if in_paths {
                let trimmed = line.trim();
                if let Some(prefix) = trimmed.strip_prefix("- ") {
                    out.push((prefix.trim().to_string(), name.clone()));
                } else if !trimmed.is_empty() && !trimmed.starts_with('-') {
                    in_paths = false;
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Layered index (`.meta/pointers.yaml` + `.meta/pointers/<first2>/` detail files)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct IndexEntry {
    pub uuid: String,
    pub token: String,
    pub kind: String,
    pub file: String,
    pub line_start: usize,
    pub line_end: usize,
    pub checksum: u64,
    pub meta_ref: String,
    pub component: String,
    // detail fields
    pub name: String,
    pub dtype: String,
    pub description: String,
    pub language: String,
    pub notes: String,
    pub change_history: Vec<String>,
}

pub fn fnv1a64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Write the slim master index and sharded detail files. Returns count written.
pub fn write_index(root: &Path, entries: &[IndexEntry]) -> Result<usize, String> {
    let mut sorted: Vec<&IndexEntry> = entries.iter().collect();
    sorted.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    let mut yaml = String::from("# generated by doc-pointers — do not edit\npointers:\n");
    for e in &sorted {
        yaml.push_str(&format!(
            "  - uuid: {}\n    token: {}\n    kind: {}\n    file: {}\n    line_start: {}\n    line_end: {}\n    checksum: {:016x}\n    meta_ref: {}\n    component: {}\n",
            e.uuid, e.token, e.kind, e.file, e.line_start, e.line_end, e.checksum,
            e.meta_ref, yaml_str(&e.component)
        ));
    }
    let meta_dir = root.join(".meta");
    fs::create_dir_all(&meta_dir)
        .map_err(|e| format!("could not create {}: {e}", meta_dir.display()))?;
    fs::write(meta_dir.join("pointers.yaml"), yaml)
        .map_err(|e| format!("could not write pointers.yaml: {e}"))?;

    for e in &sorted {
        let shard = &e.uuid[..2.min(e.uuid.len())];
        let dir = root.join(".meta/pointers").join(shard);
        fs::create_dir_all(&dir)
            .map_err(|e| format!("could not create {}: {e}", dir.display()))?;
        let mut detail = String::new();
        detail.push_str(&format!("name: {}\n", yaml_str(&e.name)));
        detail.push_str(&format!("type: {}\n", e.kind));
        detail.push_str(&format!("description: {}\n", yaml_str(&e.description)));
        detail.push_str(&format!("language: {}\n", e.language));
        detail.push_str(&format!("notes: {}\n", yaml_str(&e.notes)));
        detail.push_str("change_history:\n");
        for h in &e.change_history {
            detail.push_str(&format!("  - {}\n", yaml_str(h)));
        }
        fs::write(dir.join(format!("{}.yaml", e.uuid)), detail)
            .map_err(|e| format!("could not write detail file: {e}"))?;
    }
    Ok(sorted.len())
}

/// Parse the slim master index written by `write_index`.
pub fn read_index(root: &Path) -> Result<Vec<IndexEntry>, String> {
    let text = fs::read_to_string(root.join(".meta/pointers.yaml"))
        .map_err(|e| format!("could not read .meta/pointers.yaml: {e}"))?;
    let mut entries = Vec::new();
    let mut current: Option<IndexEntry> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "pointers:" || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with("- ") {
            if let Some(c) = current.take() {
                entries.push(c);
            }
            let rest = trimmed[2..].trim();
            if let Some((k, v)) = rest.split_once(": ") {
                let mut entry = IndexEntry::default();
                set_index_field(&mut entry, k, v);
                current = Some(entry);
            }
            continue;
        }
        if let Some(entry) = current.as_mut() {
            if let Some((k, v)) = trimmed.split_once(": ") {
                set_index_field(entry, k, v);
            }
        }
    }
    if let Some(c) = current.take() {
        entries.push(c);
    }
    Ok(entries)
}

fn set_index_field(entry: &mut IndexEntry, key: &str, value: &str) {
    match key {
        "uuid" => entry.uuid = value.trim_matches('"').to_string(),
        "token" => entry.token = value.trim_matches('"').to_string(),
        "kind" => entry.kind = value.to_string(),
        "file" => entry.file = value.trim_matches('"').to_string(),
        "line_start" => entry.line_start = value.parse().unwrap_or(0),
        "line_end" => entry.line_end = value.parse().unwrap_or(0),
        "checksum" => entry.checksum = u64::from_str_radix(value, 16).unwrap_or(0),
        "meta_ref" => entry.meta_ref = value.trim_matches('"').to_string(),
        "component" => entry.component = value.trim_matches('"').to_string(),
        _ => {}
    }
}

fn yaml_str(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    if value.contains(':') || value.contains('#') || value.contains('"') {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

// ---------------------------------------------------------------------------
// Public command implementations (wired from the bin)
// ---------------------------------------------------------------------------

/// Collect all prefixed declarations + logic blocks across the tree.
pub fn collect_prefixed(
    root: &Path,
    files: &[PathBuf],
    strict: bool,
) -> Result<(Vec<Decl>, Vec<String>), String> {
    let mut decls = Vec::new();
    let mut errors = Vec::new();
    let mut cross_file_opens: HashMap<String, Decl> = HashMap::new();
    for file in files {
        let Ok(text) = fs::read_to_string(file) else {
            continue;
        };
        let uuid_of = |seed: &str| uuid5_str(seed);
        let (mut d, mut e) = scan_file(root, file, &text, &uuid_of);
        for decl in d.drain(..) {
            if decl.kind == Kind::Logic && decl.close_line.is_none() {
                cross_file_opens.insert(decl.code.clone(), decl.clone());
            }
            decls.push(decl);
        }
        errors.append(&mut e);
    }
    // Second pass: close lines for blocks whose open was in another file.
    for file in files {
        let Ok(text) = fs::read_to_string(file) else {
            continue;
        };
        let relfile = rel_path(file, root);
        for (idx, line) in text.lines().enumerate() {
            let Some(code) = parse_logic_close(line) else {
                continue;
            };
            if let Some(decl) = decls.iter_mut().find(|d| d.code == code && d.close_line.is_none())
            {
                decl.line_end = idx + 1;
                decl.close_file = Some(decl.file.clone());
                decl.close_line = Some(idx + 1);
                if decl.file != relfile {
                    decl.file = format!("{} -> {}", decl.file, relfile);
                }
                cross_file_opens.remove(&code);
            }
        }
    }
    for (code, decl) in &cross_file_opens {
        let msg = format!(
            "logic block {code} opened at {}:{} is never closed anywhere",
            decl.file, decl.line_start
        );
        if strict {
            errors.push(msg);
        }
    }
    // Duplicate code check.
    let mut seen: HashMap<&str, &Decl> = HashMap::new();
    for decl in &decls {
        if let Some(first) = seen.get(decl.code.as_str()) {
            errors.push(format!(
                "duplicate pointer {}: {}:{} and {}:{}",
                decl.code, first.file, first.line_start, decl.file, decl.line_start
            ));
        } else {
            seen.insert(decl.code.as_str(), decl);
        }
    }
    Ok((decls, errors))
}

/// Build the layered index from a scan.
pub fn build_index(root: &Path, files: &[PathBuf]) -> Result<Vec<IndexEntry>, String> {
    let (decls, errors) = collect_prefixed(root, files, false)?;
    for e in &errors {
        eprintln!("WARNING: {e}");
    }
    let components = load_components(root);
    let mut entries = Vec::new();
    for decl in decls {
        let abs = root.join(&decl.file);
        let checksum = fs::read(&abs).map(|data| fnv1a64(&data)).unwrap_or(0);
        let language = lang_for_path(&decl.file).map(lang_name).unwrap_or("text");
        let component = components
            .iter()
            .find(|(prefix, _)| decl.file.starts_with(prefix.as_str()))
            .map(|(_, c)| c.clone())
            .unwrap_or_default();
        entries.push(IndexEntry {
            uuid: decl.uuid.clone(),
            token: decl.code.clone(),
            kind: decl.kind.name().to_string(),
            file: decl.file.clone(),
            line_start: decl.line_start,
            line_end: decl.line_end,
            checksum,
            meta_ref: format!(".meta/pointers/{}/{}.yaml", &decl.uuid[..2], decl.uuid),
            component,
            name: decl.name,
            dtype: decl.kind.name().to_string(),
            description: decl.description,
            language: language.to_string(),
            notes: String::new(),
            change_history: Vec::new(),
        });
    }
    write_index(root, &entries)?;
    Ok(entries)
}

/// Resolve a token or uuid against the index, then walk parents:
/// logic ← enclosing function ← enclosing module ← file ← component.
pub fn get_chain(root: &Path, key: &str) -> Result<Vec<IndexEntry>, String> {
    let entries = read_index(root)?;
    let token = key.trim_start_matches("⟦").trim_end_matches('⟧');
    let entry = entries
        .iter()
        .find(|e| e.token == token || e.uuid == key)
        .cloned()
        .ok_or_else(|| format!("pointer {key} not found in .meta/pointers.yaml"))?;
    let mut chain = vec![entry.clone()];
    // Parents by containment in the same file (compare on the pre-arrow file path).
    let base_file = entry
        .file
        .split(" -> ")
        .next()
        .unwrap_or(&entry.file)
        .to_string();
    let is_ancestor = |parent: &IndexEntry, child: &IndexEntry| {
        parent
            .file
            .split(" -> ")
            .next()
            .map(|f| f == base_file)
            .unwrap_or(false)
            && parent.line_start <= child.line_start
            && parent.line_end >= child.line_end
    };
    for kind in ["function", "module", "file"] {
        if let Some(parent) = entries
            .iter()
            .filter(|e| e.kind == kind && e.uuid != entry.uuid)
            .find(|e| is_ancestor(e, &entry))
        {
            chain.push(parent.clone());
        }
    }
    if !entry.component.is_empty() {
        // Component is not a pointer row; synthesize a stub so the chain is uniform.
        chain.push(IndexEntry {
            kind: "component".to_string(),
            name: entry.component.clone(),
            ..IndexEntry::default()
        });
    }
    Ok(chain)
}

/// Recompute path/range/checksum for one pointer (by token or uuid) and rewrite the index.
pub fn update_pointer(root: &Path, key: &str) -> Result<IndexEntry, String> {
    let mut entries = read_index(root)?;
    let token = key.trim_start_matches('⟦').trim_end_matches('⟧');
    let pos = entries
        .iter()
        .position(|e| e.token == token || e.uuid == key)
        .ok_or_else(|| format!("pointer {key} not found"))?;
    let old = entries[pos].clone();
    let base_file = old.file.split(" -> ").next().unwrap_or(&old.file).to_string();
    let abs = root.join(&base_file);
    let data = fs::read(&abs).map_err(|e| format!("could not read {}: {e}", abs.display()))?;
    let checksum = fnv1a64(&data);
    let text = String::from_utf8_lossy(&data).to_string();
    let lines: Vec<&str> = text.lines().collect();
    // Re-locate the marker by code.
    let mut new_start = old.line_start;
    let mut new_end = old.line_end;
    let mut found = false;
    for (idx, line) in lines.iter().enumerate() {
        let has = match old.kind.as_str() {
            "logic" => parse_logic_open(line)
                .map(|(c, _, _)| c == old.token)
                .unwrap_or(false)
                || parse_logic_close(line)
                    .map(|c| c == old.token)
                    .unwrap_or(false),
            _ => parse_prefixed(line)
                .map(|(_, c, _, _)| c == old.token)
                .unwrap_or(false),
        };
        if has {
            found = true;
            new_start = idx + 1;
            if old.kind == "file" {
                new_end = lines.len();
            } else if old.kind == "logic" {
                // find the close
                new_end = idx + 1;
                for (jdx, jline) in lines.iter().enumerate().skip(idx + 1) {
                    if parse_logic_close(jline).map(|c| c == old.token).unwrap_or(false) {
                        new_end = jdx + 1;
                        break;
                    }
                }
            } else if let Some(lang) = lang_for_path(&base_file) {
                new_end = naive_range(lang, &lines, idx + 1);
            } else {
                new_end = idx + 1;
            }
            break;
        }
    }
    if !found {
        return Err(format!(
            "marker {} no longer present in {}",
            old.token, base_file
        ));
    }
    entries[pos].file = base_file;
    entries[pos].line_start = new_start;
    entries[pos].line_end = new_end;
    entries[pos].checksum = checksum;
    write_index(root, &entries)?;
    Ok(entries[pos].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_emoji_and_ascii_prefixes() {
        for (glyph, ascii, kind) in [
            ("📁", "@f:", Kind::File),
            ("🧩", "@m:", Kind::Module),
            ("🔧", "@fn:", Kind::Function),
            ("🔀", "@l:", Kind::Logic),
        ] {
            let line = format!("// {glyph}⟦ABCD⟧ Thing :: Does the thing.");
            let (k, code, name, desc) = parse_prefixed(&line).unwrap();
            assert_eq!(k, kind);
            assert_eq!(code, "ABCD");
            assert_eq!(name, "Thing");
            assert_eq!(desc, "Does the thing.");
            let line = format!("// {ascii}⟦ABCD⟧ Thing :: Does the thing.");
            let (k, code, _, _) = parse_prefixed(&line).unwrap();
            assert_eq!(k, kind);
            assert_eq!(code, "ABCD");
        }
        // Legacy unprefixed marker must NOT parse as prefixed.
        assert!(parse_prefixed("// ⟦ABCD⟧ Legacy :: no prefix").is_none());
    }

    #[test]
    fn parses_logic_block_open_close() {
        let (code, name, desc) = parse_logic_open("// [[🔀:ABCD]] Retry loop :: Backoff logic").unwrap();
        assert_eq!(code, "ABCD");
        assert_eq!(name, "Retry loop");
        assert_eq!(desc, "Backoff logic");
        assert_eq!(parse_logic_close("// [[/🔀:ABCD]]").unwrap(), "ABCD");
        assert!(parse_logic_close("// [[🔀:ABCD]] open not close").is_none());
    }

    #[test]
    fn same_file_block_gets_exact_range() {
        let root = std::env::temp_dir().join(format!("dp-rt-{}", uuid::Uuid::new_v4()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        let body = "defmodule Demo do\n  def a do\n    [[🔀:ABCD]] Loop :: retry\n    :ok\n    [[/🔀:ABCD]]\n  end\nend\n";
        fs::write(root.join("src/demo.ex"), body).unwrap();
        let (decls, errors) = collect_prefixed(&root, &[root.join("src/demo.ex")], true).unwrap();
        assert!(errors.is_empty(), "{errors:?}");
        let block = decls.iter().find(|d| d.kind == Kind::Logic).unwrap();
        assert_eq!(block.line_start, 3);
        assert_eq!(block.line_end, 5);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn unterminated_block_errors_under_strict_only() {
        let root = std::env::temp_dir().join(format!("dp-rt-{}", uuid::Uuid::new_v4()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/demo.ex"), "def a do\n  [[🔀:ABCD]] Loop :: x\n  :ok\nend\n").unwrap();
        let files = vec![root.join("src/demo.ex")];
        let (_, lax) = collect_prefixed(&root, &files, false).unwrap();
        assert!(!lax.iter().any(|e| e.contains("never closed")));
        let (_, strict) = collect_prefixed(&root, &files, true).unwrap();
        assert!(strict.iter().any(|e| e.contains("never closed")));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn naive_ranges_elixir_and_rust() {
        let ex: Vec<&str> = vec!["defmodule M do", "  def f do", "    :ok", "  end", "end"];
        assert_eq!(naive_range(Lang::Elixir, &ex, 1), 5); // module covers whole file
        assert_eq!(naive_range(Lang::Elixir, &ex, 2), 4); // function ends at its end
        let rs: Vec<&str> = vec!["pub fn f() {", "    g();", "}"];
        assert_eq!(naive_range(Lang::Rust, &rs, 1), 3);
    }

    #[test]
    fn parse_diff_extracts_new_side_hunks() {
        let diff = "\
diff --git a/src/a.ex b/src/a.ex
--- a/src/a.ex
+++ b/src/a.ex
@@ -10,7 +10,9 @@ defmodule A do
 unchanged
+added
 unchanged
@@ -40,3 +42,1 @@
-removed
 context
diff --git a/src/gone.ex b/src/gone.ex
--- a/src/gone.ex
+++ /dev/null
@@ -1,5 +0,0 @@
-x";
        let files = parse_diff(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "src/a.ex");
        assert_eq!(files[0].hunks, vec![(10, 9), (42, 1)]);
    }

    #[test]
    fn impact_reports_overlapping_pointer_only() {
        let root = std::env::temp_dir().join(format!("dp-rt-{}", uuid::Uuid::new_v4()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/demo.ex"),
            "# @fn:⟦FNC1⟧ hitter :: touched\ndef hit do\n  :ok\nend\n\n# @fn:⟦FNC2⟧ safe :: untouched\ndef safe do\n  :ok\nend\n",
        )
        .unwrap();
        let diff = "diff --git a/src/demo.ex b/src/demo.ex\n--- a/src/demo.ex\n+++ b/src/demo.ex\n@@ -1,3 +1,4 @@\n # @fn:⟦FNC1⟧ hitter :: touched\n+extra\n def hit do\n";
        let hits = impact(&root, &diff, &[]).unwrap();
        let codes: Vec<&str> = hits.iter().map(|h| h.code.as_str()).collect();
        assert!(codes.contains(&"FNC1"), "{codes:?}");
        assert!(!codes.contains(&"FNC2"), "{codes:?}");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn layered_index_roundtrip() {
        let root = std::env::temp_dir().join(format!("dp-rt-{}", uuid::Uuid::new_v4()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let entry = IndexEntry {
            uuid: "5c692577-ad0c-51f1-992c-759b5e5fffb5".to_string(),
            token: "ABCD".to_string(),
            kind: "function".to_string(),
            file: "src/lib.rs".to_string(),
            line_start: 10,
            line_end: 20,
            checksum: 0xdeadbeef,
            meta_ref: ".meta/pointers/5c/5c692577-ad0c-51f1-992c-759b5e5fffb5.yaml".to_string(),
            component: "core".to_string(),
            name: "thing".to_string(),
            dtype: "function".to_string(),
            description: "a: thing".to_string(),
            language: "rust".to_string(),
            notes: String::new(),
            change_history: vec![],
        };
        write_index(&root, &[entry.clone()]).unwrap();
        let read = read_index(&root).unwrap();
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].uuid, entry.uuid);
        assert_eq!(read[0].checksum, 0xdeadbeef);
        assert_eq!(read[0].line_start, 10);
        assert_eq!(read[0].line_end, 20);
        assert_eq!(read[0].component, "core");
        assert!(root.join(".meta/pointers/5c/5c692577-ad0c-51f1-992c-759b5e5fffb5.yaml").exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn uuid5_is_deterministic_per_seed() {
        assert_eq!(uuid5_str("src/a.ex::#loop"), uuid5_str("src/a.ex::#loop"));
        assert_ne!(uuid5_str("src/a.ex::#loop"), uuid5_str("src/a.ex::#other"));
    }
}

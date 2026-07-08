use std::collections::{HashMap, HashSet};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use uuid::Uuid;

const DEFAULT_DB_PATH: &str = "docs/doc-pointer-db.json";
const DOC_POINTER_NAMESPACE: Uuid = uuid::uuid!("64e9408c-37a7-5f92-8893-f149cbde01c0");
const TOKEN_START: u32 = 0x13000;
const TOKEN_END: u32 = 0x1342F;
const TOKEN_SIZE: u128 = (TOKEN_END - TOKEN_START + 1) as u128;
const TOKEN_LENGTH: usize = 4;

#[derive(Debug, Clone)]
struct Pointer {
    code: String,
    path: String,
    line: usize,
    name: String,
    description: String,
}

#[derive(Debug)]
struct ScanOptions {
    root: PathBuf,
    db: String,
    write: bool,
    check: bool,
    install_hook: bool,
}

#[derive(Debug)]
struct Uuid5Options {
    name: Option<String>,
    root: PathBuf,
    db: String,
    namespace: String,
    salt: String,
    format: PointerFormat,
    description: String,
    no_clipboard: bool,
}

#[derive(Debug, Clone, Copy)]
enum PointerFormat {
    Marker,
    Code,
    Declaration,
    Deeplink,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // No arguments at all -> print help and exit 0. Previously this silently ran a
    // read-only scan, which surprised users who just wanted to know what the tool does.
    if args.is_empty() {
        print_help();
        std::process::exit(0);
    }

    // First positional token selects the subcommand (build / uuid5 / hook / help).
    // A leading option (-h/--help, or a legacy build flag like --root/--write/--check)
    // is treated as the build command so existing scripts keep working.
    let result = match args.first().map(String::as_str) {
        Some("build") => scan_command(&args[1..]),
        Some("uuid5" | "new") => uuid5_command(&args[1..]),
        Some("hook") => install_command(&args[1..], true),
        Some("-h" | "--help" | "help") => {
            print_help();
            return;
        }
        Some("--root" | "--db" | "--write" | "--check" | "--install-hook") => {
            scan_or_install(&args)
        }
        Some(value) => {
            eprintln!("ERROR: unknown subcommand or option: {value}\n");
            print_help();
            std::process::exit(1);
        }
        None => {
            print_help();
            return;
        }
    };

    if let Err(error) = result {
        eprintln!("ERROR: {error}");
        std::process::exit(1);
    }
}

// Legacy dispatch: the build/install-hook commands used to share one option namespace at the
// top level. `--install-hook` installs the pre-commit hook; everything else is a build scan.
fn scan_or_install(args: &[String]) -> Result<(), String> {
    if args.iter().any(|arg| arg == "--install-hook") {
        install_command(args, false)
    } else {
        scan_command(args)
    }
}

fn install_command(args: &[String], explicit: bool) -> Result<(), String> {
    let options = parse_scan_options(args)?;
    // `explicit` = reached via the `hook` subcommand (always install). The legacy
    // `--install-hook` flag also sets options.install_hook; reject ambiguous calls.
    if !explicit && !options.install_hook {
        eprintln!("ERROR: --install-hook requires the hook subcommand or the --install-hook flag");
        std::process::exit(1);
    }
    let root = absolute_path(&options.root)?;
    install_hook(&root)
}

fn scan_command(args: &[String]) -> Result<(), String> {
    let options = parse_scan_options(args)?;
    let root = absolute_path(&options.root)?;
    let db_path = absolute_path(&root.join(&options.db))?;

    let (pointers, mut errors) = collect_pointers(&root, &db_path)?;
    let (changed_links, link_errors) = expand_markdown_links(&root, &pointers, options.write)?;
    errors.extend(link_errors);

    let mut db_changed = false;
    if options.write {
        db_changed = write_db(&root, &db_path, &pointers)?;
    } else if options.check {
        let expected = db_payload(&pointers);
        let existing = fs::read_to_string(&db_path).ok();
        db_changed = existing.as_deref() != Some(expected.as_str());
    }

    for error in &errors {
        eprintln!("ERROR: {error}");
    }

    if options.check {
        if db_changed {
            eprintln!(
                "ERROR: {} is stale; run make doc-pointers.",
                rel_path(&db_path, &root)
            );
        }
        if !changed_links.is_empty() {
            eprintln!(
                "ERROR: markdown deeplinks need expansion in: {}",
                changed_links.join(", ")
            );
        }
        if !errors.is_empty() || db_changed || !changed_links.is_empty() {
            std::process::exit(1);
        }
    }

    if options.write {
        if db_changed {
            println!("updated {}", rel_path(&db_path, &root));
        }
        if !changed_links.is_empty() {
            println!(
                "expanded markdown deeplinks in: {}",
                changed_links.join(", ")
            );
        }
        if !db_changed && changed_links.is_empty() {
            println!("doc pointers already up to date");
        }
        if !errors.is_empty() {
            std::process::exit(1);
        }
    } else {
        println!("found {} doc pointer declarations", pointers.len());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

fn uuid5_command(args: &[String]) -> Result<(), String> {
    let options = parse_uuid5_options(args)?;
    let root = absolute_path(&options.root)?;
    let db_path = absolute_path(&root.join(&options.db))?;
    let namespace = parse_namespace(&options.namespace)?;
    let (pointers, errors) = collect_pointers(&root, &db_path)?;

    for error in errors {
        eprintln!("WARNING: {error}");
    }

    let seed = options
        .name
        .clone()
        .unwrap_or_else(|| format!("auto:{}", Uuid::new_v4()));
    let (code, uuid, uuid_name, attempt) =
        generate_uuid5_code(&seed, namespace, &options.salt, &pointers)?;
    let payload = format_pointer(
        options.format,
        &code,
        options.name.as_deref(),
        &options.description,
    )?;

    println!("uuid5: {uuid}");
    println!("uuid5-name: {uuid_name}");
    if attempt > 0 {
        println!("collision-attempt: {attempt}");
    }
    println!("code: {code}");
    println!("marker: ⟦{code}⟧");
    println!("clipboard: {payload}");

    if !options.no_clipboard {
        copy_to_clipboard(&payload)?;
        println!("copied to clipboard");
    }

    Ok(())
}

fn parse_scan_options(args: &[String]) -> Result<ScanOptions, String> {
    let mut options = ScanOptions {
        root: PathBuf::from("."),
        db: DEFAULT_DB_PATH.to_string(),
        write: false,
        check: false,
        install_hook: false,
    };

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "-h" | "--help" => {
                print_scan_help();
                std::process::exit(0);
            }
            "--root" => {
                index += 1;
                options.root = PathBuf::from(expect_value(args, index, "--root")?);
            }
            "--db" => {
                index += 1;
                options.db = expect_value(args, index, "--db")?.to_string();
            }
            "--write" => options.write = true,
            "--check" => options.check = true,
            "--install-hook" => options.install_hook = true,
            value => return Err(format!("unknown option: {value}")),
        }
        index += 1;
    }

    Ok(options)
}

fn parse_uuid5_options(args: &[String]) -> Result<Uuid5Options, String> {
    let mut name = None;
    let mut root = PathBuf::from(".");
    let mut db = DEFAULT_DB_PATH.to_string();
    let mut namespace = "doc-pointers".to_string();
    let mut salt = String::new();
    let mut format = PointerFormat::Marker;
    let mut description = String::new();
    let mut no_clipboard = false;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "-h" | "--help" => {
                print_uuid5_help();
                std::process::exit(0);
            }
            "--root" => {
                index += 1;
                root = PathBuf::from(expect_value(args, index, "--root")?);
            }
            "--db" => {
                index += 1;
                db = expect_value(args, index, "--db")?.to_string();
            }
            "--namespace" => {
                index += 1;
                namespace = expect_value(args, index, "--namespace")?.to_string();
            }
            "--salt" => {
                index += 1;
                salt = expect_value(args, index, "--salt")?.to_string();
            }
            "--format" => {
                index += 1;
                format = match expect_value(args, index, "--format")? {
                    "marker" => PointerFormat::Marker,
                    "code" => PointerFormat::Code,
                    "declaration" => PointerFormat::Declaration,
                    "deeplink" => PointerFormat::Deeplink,
                    value => return Err(format!("invalid --format value: {value}")),
                };
            }
            "--description" => {
                index += 1;
                description = expect_value(args, index, "--description")?.to_string();
            }
            "--no-clipboard" => no_clipboard = true,
            value if value.starts_with('-') => return Err(format!("unknown option: {value}")),
            value => {
                if name.is_some() {
                    return Err(format!("unexpected argument: {value}"));
                }
                name = Some(value.to_string());
            }
        }
        index += 1;
    }

    Ok(Uuid5Options {
        name,
        root,
        db,
        namespace,
        salt,
        format,
        description,
        no_clipboard,
    })
}

fn expect_value<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, String> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn print_help() {
    println!(
        "\
doc-pointers — durable, code-stable cross-document pointers

WHAT
  Doc pointers are 4-character tokens (drawn from the Unicode U+13000..U+1342F
  Egyptian Hieroglyphs block) placed as inline declarations inside source files,
  comments, and Markdown. A declaration looks like:

      ⟦𓆴𓎲𓋝𓁅⟧ Pointer name :: Human readable description.

  Because the token is a fixed Unicode glyph sequence and not a path or line
  number, it survives renames, refactors, and file moves. Other documents then
  reference it with a `deeplink:` Markdown link, e.g.

      [see routing](deeplink:⟦𓆴𓎲𓋝𓁅⟧)

  and `doc-pointers` rewrites that link into a concrete `path:line?code=⟦…⟧` target.

HOW
  1. Place a ⟦token⟧ Name :: Description declaration at the anchor you want to
     point at (a function, a heading, a config stanza). The token must live in a
     comment context (//, #, <!--, /*, *, --, ;) or on its own line, so it never
     collides with string literals or code.
  2. Generate new tokens with `doc-pointers uuid5` (deterministic UUIDv5,
     collision-checked against the existing database, copied to your clipboard).
  3. Reference any token from Markdown with `[label](deeplink:⟦token⟧)`.
  4. Run `doc-pointers build` to (a) collect every declaration in the repo into
     `docs/doc-pointer-db.json` and (b) expand every `deeplink:` reference into a
     real `file:line` target. Run `doc-pointers build --check` in CI / a
     pre-commit hook to fail when the database or any link is stale.

WHY
  Ordinary file:line and URL links rot the instant code moves. Branch names and
  permalinks are worse. A doc pointer decouples the *identity* of an anchor
  (the token) from its *current location* (which `build` resolves on demand), so
  docs stay accurate without manual re-pointing. The Hieroglyph block is chosen
  so tokens are visually distinct and never appear in real source, and the
  UUIDv5 derivation makes them reproducible and collision-free across machines.

USAGE
  doc-pointers                       print this help
  doc-pointers build                 collect declarations + expand deeplinks (read-only)
  doc-pointers build --write         also write docs/doc-pointer-db.json + expanded links
  doc-pointers build --check         fail (exit 1) if a write would change anything
  doc-pointers hook                  install a pre-commit hook that runs --check
  doc-pointers uuid5 [NAME]          mint a new 4-glyph token, copied to clipboard
  doc-pointers help                  print this help

SUB-COMMAND HELP
  doc-pointers build --help          options for the build/scan command
  doc-pointers uuid5 --help          options for the uuid5 command

LEGACY
  The bare flags still work for existing scripts:
    doc-pointers --write        ==  doc-pointers build --write
    doc-pointers --check        ==  doc-pointers build --check
    doc-pointers --install-hook ==  doc-pointers hook
  Prefer the subcommand form in new code."
    );
}

fn print_scan_help() {
    println!(
        "usage: doc-pointers build [--root ROOT] [--db DB] [--write] [--check]\n\n\
Build the doc pointer database and expand deeplink: markdown links.\n\
Run without --write/--check, this is a dry run: it reports how many\n\
declarations it found and any errors, but changes nothing.\n\n\
options:\n  --root ROOT       repository root, default: current directory\n  --db DB           pointer database path (default docs/doc-pointer-db.json)\n  --write           write database and expand markdown deeplinks\n  --check           fail if writes would be needed (CI / pre-commit)"
    );
}

fn print_uuid5_help() {
    println!(
        "usage: doc-pointers uuid5 [options] [NAME]\n\n\
Generate a deterministic UUIDv5-backed four-character doc pointer token.\n\
The token is collision-checked against the current database and copied to\n\
the clipboard unless --no-clipboard is given.\n\n\
options:\n  --root ROOT             repository root, default: current directory\n  --db DB                 pointer database path\n  --namespace NAMESPACE   doc-pointers, dns, url, oid, x500, or a UUID\n  --salt SALT             optional deterministic salt\n  --format FORMAT         marker, code, declaration, or deeplink\n  --description TEXT      description used by --format declaration\n  --no-clipboard          print without copying to clipboard"
    );
}

fn collect_pointers(
    root: &Path,
    db_path: &Path,
) -> Result<(HashMap<String, Pointer>, Vec<String>), String> {
    let mut pointers: HashMap<String, Pointer> = HashMap::new();
    let mut errors = Vec::new();
    for path in scan_files(root, db_path)? {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let mut in_fence = false;
        for (index, line) in text.lines().enumerate() {
            if path.extension() == Some(OsStr::new("md")) && line.trim_start().starts_with("```") {
                in_fence = !in_fence;
                continue;
            }
            if in_fence {
                continue;
            }
            let Some((code, name, description)) = parse_declaration(line) else {
                continue;
            };
            let pointer = Pointer {
                code: code.clone(),
                path: rel_path(&path, root),
                line: index + 1,
                name,
                description,
            };
            if let Some(first) = pointers.get(&code) {
                errors.push(format!(
                    "duplicate pointer {code}: {}:{} and {}:{}",
                    first.path, first.line, pointer.path, pointer.line
                ));
            } else {
                pointers.insert(code, pointer);
            }
        }
    }
    Ok((pointers, errors))
}

fn parse_declaration(line: &str) -> Option<(String, String, String)> {
    let start = line.find('⟦')?;
    if !declaration_context_allows(line, start) {
        return None;
    }
    let after_start = start + '⟦'.len_utf8();
    let end_offset = line[after_start..].find('⟧')?;
    let end = after_start + end_offset;
    let code = &line[after_start..end];
    if code.chars().count() != 4 || !valid_code(code) {
        return None;
    }
    let rest = line[end + '⟧'.len_utf8()..].trim_start();
    let (name, description) = rest.split_once("::")?;
    let name = clean_comment_tail(name);
    if name.is_empty() {
        return None;
    }
    Some((code.to_string(), name, clean_comment_tail(description)))
}

fn declaration_context_allows(line: &str, start: usize) -> bool {
    let prefix = &line[..start];
    let before = prefix.trim_end();
    if before.is_empty() {
        return true;
    }

    let trimmed = before.trim_start();
    let leading_comment_markers = ["//", "#", "<!--", "/*", "*", "--", ";"];
    if leading_comment_markers
        .iter()
        .any(|marker| trimmed.starts_with(marker))
    {
        return true;
    }

    let inline_comment_markers = ["//", "/*", "<!--", " #", "\t#", " --"];
    if inline_comment_markers
        .iter()
        .any(|marker| trimmed.contains(marker))
    {
        return true;
    }

    false
}

fn clean_comment_tail(value: &str) -> String {
    value
        .trim()
        .trim_end_matches("-->")
        .trim_end_matches("*/")
        .trim()
        .to_string()
}

fn valid_code(code: &str) -> bool {
    code.chars()
        .all(|ch| !ch.is_whitespace() && !"⟦⟧/?#:%".contains(ch))
}

fn scan_files(root: &Path, db_path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let skip_dirs: HashSet<&str> = [
        ".DS_Store",
        ".Spotlight-V100",
        ".Trashes",
        ".fseventsd",
        ".git",
        ".idea",
        ".vscode",
        "Builds",
        "DerivedData",
        "Library",
        "Logs",
        "Temp",
        "UserSettings",
        "build",
        "coverage",
        "dist",
        "node_modules",
        "obj",
        "target",
    ]
    .into_iter()
    .collect();
    let suffixes: HashSet<&str> = [
        "asmdef", "cs", "css", "html", "json", "md", "meta", "shader", "txt", "uxml", "yaml", "yml",
    ]
    .into_iter()
    .collect();
    walk_scan(root, root, db_path, &skip_dirs, &suffixes, &mut files)?;
    Ok(files)
}

fn walk_scan(
    root: &Path,
    dir: &Path,
    db_path: &Path,
    skip_dirs: &HashSet<&str>,
    suffixes: &HashSet<&str>,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => return Err(format!("could not read {}: {error}", dir.display())),
    };
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_dir() {
            let name = entry.file_name();
            if !skip_dirs.contains(name.to_string_lossy().as_ref()) {
                walk_scan(root, &path, db_path, skip_dirs, suffixes, files)?;
            }
            continue;
        }
        if !file_type.is_file() || absolute_path(&path)? == db_path {
            continue;
        }
        if path.file_name() == Some(OsStr::new("packages-lock.json")) {
            continue;
        }
        if path
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| suffixes.contains(extension))
            && path.starts_with(root)
        {
            files.push(path);
        }
    }
    Ok(())
}

fn expand_markdown_links(
    root: &Path,
    pointers: &HashMap<String, Pointer>,
    write: bool,
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut changed = Vec::new();
    let mut errors = Vec::new();
    for path in scan_files(root, &root.join("__no_db__"))? {
        if path.extension() != Some(OsStr::new("md")) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let mut new_text = String::new();
        let mut in_fence = false;
        for line in text.split_inclusive('\n') {
            if line.trim_start().starts_with("```") {
                in_fence = !in_fence;
                new_text.push_str(line);
            } else if in_fence {
                new_text.push_str(line);
            } else {
                new_text.push_str(&expand_line(root, &path, line, pointers, &mut errors));
            }
        }
        if new_text != text {
            changed.push(rel_path(&path, root));
            if write {
                fs::write(&path, new_text)
                    .map_err(|error| format!("could not write {}: {error}", path.display()))?;
            }
        }
    }
    Ok((changed, errors))
}

fn expand_line(
    root: &Path,
    path: &Path,
    line: &str,
    pointers: &HashMap<String, Pointer>,
    errors: &mut Vec<String>,
) -> String {
    let mut output = String::new();
    let mut remaining = line;
    while let Some(start) = remaining.find("](deeplink:") {
        let before = &remaining[..start];
        let after_prefix = &remaining[start + "](deeplink:".len()..];
        let Some(end) = after_prefix.find(')') else {
            output.push_str(remaining);
            return output;
        };
        let raw_code = &after_prefix[..end];
        let code = normalize_code(raw_code);
        if code.chars().count() == 4 && valid_code(&code) {
            output.push_str(before);
            if let Some(pointer) = pointers.get(&code) {
                output.push_str(&format!("]({})", expanded_target(pointer)));
            } else {
                errors.push(format!(
                    "{}: unresolved deeplink:{code}",
                    rel_path(path, root)
                ));
                output.push_str("](deeplink:");
                output.push_str(raw_code);
                output.push(')');
            }
            remaining = &after_prefix[end + 1..];
        } else {
            output.push_str(&remaining[..start + "](deeplink:".len()]);
            remaining = after_prefix;
        }
    }
    output.push_str(remaining);
    output
}

fn normalize_code(raw: &str) -> String {
    raw.strip_prefix('⟦')
        .and_then(|value| value.strip_suffix('⟧'))
        .unwrap_or(raw)
        .to_string()
}

fn expanded_target(pointer: &Pointer) -> String {
    format!("{}:{}?code=⟦{}⟧", pointer.path, pointer.line, pointer.code)
}

fn db_payload(pointers: &HashMap<String, Pointer>) -> String {
    let mut entries: Vec<_> = pointers.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut output = String::from("{\n");
    for (index, (code, pointer)) in entries.iter().enumerate() {
        output.push_str(&format!(
            "  \"{}\": {{\n    \"path\": \"{}\",\n    \"line\": {},\n    \"name\": \"{}\",\n    \"description\": \"{}\"\n  }}",
            json_escape(code),
            json_escape(&pointer.path),
            pointer.line,
            json_escape(&pointer.name),
            json_escape(&pointer.description)
        ));
        if index + 1 != entries.len() {
            output.push(',');
        }
        output.push('\n');
    }
    output.push_str("}\n");
    output
}

fn write_db(
    root: &Path,
    db_path: &Path,
    pointers: &HashMap<String, Pointer>,
) -> Result<bool, String> {
    let payload = db_payload(pointers);
    let existing = fs::read_to_string(db_path).ok();
    if existing.as_deref() == Some(payload.as_str()) {
        return Ok(false);
    }
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
    }
    fs::write(db_path, payload)
        .map_err(|error| format!("could not write {}: {error}", db_path.display()))?;
    let _ = root;
    Ok(true)
}

fn install_hook(root: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--absolute-git-dir"])
        .output()
        .map_err(|error| format!("git directory not found; cannot install hook: {error}"))?;
    if !output.status.success() {
        return Err("git directory not found; cannot install hook".to_string());
    }
    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let hook = PathBuf::from(git_dir).join("hooks/pre-commit");
    let marker = "# therobotdrafts-doc-pointers";
    if hook.exists() {
        let existing = fs::read_to_string(&hook).unwrap_or_default();
        if !existing.contains(marker) {
            return Err(format!(
                "{} already exists and is not managed by this script",
                hook.display()
            ));
        }
    }
    let body = format!(
        "#!/bin/sh\n{marker}\nset -eu\ncd {}\nmake doc-pointers-check\n",
        shell_quote(root)
    );
    if let Some(parent) = hook.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&hook, body).map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&hook)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        fs::set_permissions(&hook, permissions).map_err(|error| error.to_string())?;
    }
    println!("installed {}", hook.display());
    Ok(())
}

fn parse_namespace(raw: &str) -> Result<Uuid, String> {
    match raw.to_ascii_lowercase().as_str() {
        "doc-pointers" => Ok(DOC_POINTER_NAMESPACE),
        "dns" => Ok(Uuid::NAMESPACE_DNS),
        "url" => Ok(Uuid::NAMESPACE_URL),
        "oid" => Ok(Uuid::NAMESPACE_OID),
        "x500" => Ok(Uuid::NAMESPACE_X500),
        _ => Uuid::parse_str(raw).map_err(|error| format!("invalid UUID namespace: {error}")),
    }
}

fn generate_uuid5_code(
    name: &str,
    namespace: Uuid,
    salt: &str,
    pointers: &HashMap<String, Pointer>,
) -> Result<(String, Uuid, String, usize), String> {
    for attempt in 0..10000 {
        let uuid_name = uuid5_name(name, salt, attempt);
        let value = Uuid::new_v5(&namespace, uuid_name.as_bytes());
        let code = unicode4_encode_uuid(value);
        if !pointers.contains_key(&code) {
            return Ok((code, value, uuid_name, attempt));
        }
    }
    Err("could not generate an unused pointer code after 10000 attempts".to_string())
}

fn uuid5_name(name: &str, salt: &str, attempt: usize) -> String {
    let mut parts = vec!["doc-pointers".to_string(), name.to_string()];
    if !salt.is_empty() {
        parts.push(salt.to_string());
    }
    if attempt > 0 {
        parts.push(attempt.to_string());
    }
    parts.join(":")
}

fn unicode4_encode_uuid(value: Uuid) -> String {
    let mut number = u128::from_be_bytes(*value.as_bytes());
    let mut chars = Vec::new();
    for _ in 0..TOKEN_LENGTH {
        let index = (number % TOKEN_SIZE) as u32;
        number /= TOKEN_SIZE;
        chars.push(char::from_u32(TOKEN_START + index).expect("valid token code point"));
    }
    chars.into_iter().rev().collect()
}

fn format_pointer(
    format: PointerFormat,
    code: &str,
    name: Option<&str>,
    description: &str,
) -> Result<String, String> {
    let marker = format!("⟦{code}⟧");
    let payload = match format {
        PointerFormat::Marker => marker,
        PointerFormat::Code => code.to_string(),
        PointerFormat::Deeplink => format!("deeplink:{marker}"),
        PointerFormat::Declaration => {
            let name = name.ok_or_else(|| "--format declaration requires NAME".to_string())?;
            format!("{marker} {name} :: {description}")
                .trim_end()
                .to_string()
        }
    };
    Ok(payload)
}

fn copy_to_clipboard(payload: &str) -> Result<(), String> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not run pbcopy: {error}"))?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| "could not open pbcopy stdin".to_string())?
        .write_all(payload.as_bytes())
        .map_err(|error| format!("could not write to pbcopy: {error}"))?;
    let status = child.wait().map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("pbcopy exited with status {status}"))
    }
}

fn json_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect(),
            '\n' => "\\n".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\t' => "\\t".chars().collect(),
            ch if ch.is_control() => format!("\\u{:04x}", ch as u32).chars().collect(),
            ch => vec![ch],
        })
        .collect()
}

fn shell_quote(path: &Path) -> String {
    let value = path.display().to_string();
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn rel_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        fs::canonicalize(path)
            .map_err(|error| format!("could not resolve {}: {error}", path.display()))
    } else if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        env::current_dir()
            .map_err(|error| error.to_string())
            .map(|cwd| cwd.join(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_declaration_accepts_bare_and_comment_lines() {
        assert_eq!(
            parse_declaration("⟦DPTR⟧ Documentation pointer convention :: Defines hard pointers."),
            Some((
                "DPTR".to_string(),
                "Documentation pointer convention".to_string(),
                "Defines hard pointers.".to_string()
            ))
        );
        assert_eq!(
            parse_declaration("// ⟦ABCD⟧ Pointer routing :: Centralizes pointer input."),
            Some((
                "ABCD".to_string(),
                "Pointer routing".to_string(),
                "Centralizes pointer input.".to_string()
            ))
        );
    }

    #[test]
    fn parse_declaration_ignores_quoted_prompt_examples() {
        let line = "            \"\\\"⟦𓅕𓀦𓈽𓆡⟧ Name :: uuid5:01234567-89ab-cdef-0123-456789abcdef\\\", remove that line\"";
        assert_eq!(parse_declaration(line), None);
    }

    #[test]
    fn parse_declaration_ignores_non_comment_code_prefixes() {
        assert_eq!(
            parse_declaration("let marker = \"⟦ABCD⟧ Name :: Description\";"),
            None
        );
        assert_eq!(
            parse_declaration("value * \"⟦ABCD⟧ Name :: Description\""),
            None
        );
    }

    #[test]
    fn generated_token_matches_unity_fixture() {
        let uuid = Uuid::new_v5(
            &DOC_POINTER_NAMESPACE,
            "doc-pointers:TestPointer".as_bytes(),
        );
        assert_eq!(uuid.to_string(), "5c692577-ad0c-51f1-992c-759b5e5fffb5");
        assert_eq!(unicode4_encode_uuid(uuid), "𓆴𓎲𓋝𓁅");
    }

    #[test]
    fn collect_pointers_skips_spotlight_cache() {
        let root = env::temp_dir().join(format!("doc-pointers-test-{}", Uuid::new_v4()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".Spotlight-V100/cache")).unwrap();
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(
            root.join(".Spotlight-V100/cache/noise.txt"),
            "⟦NOIS⟧ Noise :: Should be ignored.\n",
        )
        .unwrap();
        fs::write(
            root.join("docs/real.md"),
            "<!-- ⟦REAL⟧ Real pointer :: Should be indexed. -->\n",
        )
        .unwrap();

        let (pointers, errors) =
            collect_pointers(&root, &root.join("docs/doc-pointer-db.json")).unwrap();
        assert!(errors.is_empty());
        assert!(pointers.contains_key("REAL"));
        assert!(!pointers.contains_key("NOIS"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn expand_markdown_links_rewrites_deeplink_targets() {
        let root = env::temp_dir().join(format!("doc-pointers-test-{}", Uuid::new_v4()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(
            root.join("docs/target.md"),
            "<!-- ⟦ABCD⟧ Target pointer :: Used by markdown expansion. -->\n",
        )
        .unwrap();
        fs::write(
            root.join("docs/ref.md"),
            "[target](deeplink:⟦ABCD⟧)\n\n```markdown\n[ignored](deeplink:ABCD)\n```\n",
        )
        .unwrap();

        let (pointers, errors) =
            collect_pointers(&root, &root.join("docs/doc-pointer-db.json")).unwrap();
        assert!(errors.is_empty());
        let (changed, link_errors) = expand_markdown_links(&root, &pointers, true).unwrap();
        assert!(link_errors.is_empty());
        assert_eq!(changed, vec!["docs/ref.md".to_string()]);

        let rewritten = fs::read_to_string(root.join("docs/ref.md")).unwrap();
        assert!(rewritten.contains("[target](docs/target.md:1?code=⟦ABCD⟧)"));
        assert!(rewritten.contains("[ignored](deeplink:ABCD)"));

        let _ = fs::remove_dir_all(&root);
    }
}

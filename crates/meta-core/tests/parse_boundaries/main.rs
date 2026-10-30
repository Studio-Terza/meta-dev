//! The string-twiddling ban (`clippy.toml`, owner 2026-10-30) has one way out: a module-level
//! `#![allow(clippy::disallowed_methods)]` in a file registered in `self/parse-boundaries.toml`.
//! This test keeps that way out the only one. Source is read with `syn`, manifests with `toml`,
//! and both are compared as paths and components, never as text.
#![deny(clippy::disallowed_methods)]

mod source;

use meta_core::Reason;
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use serde::Deserialize;
use source::Source;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

type Files = BTreeMap<PathBuf, String>;

const REGISTRY: &str = "self/parse-boundaries.toml";
const GUARD: [&str; 2] = ["clippy", "disallowed_methods"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    #[serde(default)]
    boundary: Vec<Boundary>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Boundary {
    path: PathBuf,
    #[allow(dead_code)]
    reason: Reason,
    #[allow(dead_code)]
    source: Source,
}

fn os(name: &str) -> &OsStr {
    OsStr::new(name)
}

fn named(path: &Path, name: &str) -> bool {
    path.file_name() == Some(os(name))
}

fn is_rust(path: &Path) -> bool {
    path.extension() == Some(os("rs"))
}

// ---- reading the workspace -------------------------------------------------------------------

fn read_tree(root: &Path, directory: &Path, files: &mut Files) {
    let mut entries: Vec<_> = fs::read_dir(directory).unwrap().map(|entry| entry.unwrap().path()).collect();
    entries.sort();
    for path in entries {
        match path.is_dir() {
            true if named(&path, "target") || named(&path, ".git") => {}
            true => read_tree(root, &path, files),
            false if is_rust(&path) || named(&path, "Cargo.toml") => {
                files.insert(path.strip_prefix(root).unwrap().to_path_buf(), fs::read_to_string(&path).unwrap());
            }
            false => {}
        }
    }
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

// ---- lint attributes ---------------------------------------------------------------------------

#[derive(Debug)]
struct Hit {
    level: String,
    lint: Vec<String>,
}

impl Hit {
    fn is(&self, levels: &[&str]) -> bool {
        self.lint == GUARD && levels.contains(&self.level.as_str())
    }
}

const LEVELS: [&str; 5] = ["allow", "expect", "warn", "forbid", "deny"];

/// The lints that must stay on: the banned methods themselves and every group that could carry
/// or override them.
fn guarded(lint: &[String]) -> bool {
    let names: Vec<&str> = lint.iter().map(String::as_str).collect();
    matches!(
        names.as_slice(),
        ["clippy", "disallowed_methods"]
            | ["disallowed_methods"]
            | ["clippy", "style" | "all"]
            | ["clippy"]
            | ["warnings"]
    )
}

/// The comma-separated paths inside a token stream, however deeply grouped.
fn paths(tokens: TokenStream) -> Vec<Vec<String>> {
    let mut found = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for tree in tokens {
        match tree {
            TokenTree::Ident(ident) => current.push(ident.to_string()),
            TokenTree::Punct(punct) if punct.as_char() == ',' => found.push(std::mem::take(&mut current)),
            TokenTree::Group(group) => {
                found.push(std::mem::take(&mut current));
                found.extend(paths(group.stream()));
            }
            _ => {}
        }
    }
    found.push(current);
    found
}

/// Every `allow(..)`, `expect(..)`, `warn(..)`, `forbid(..)` or `deny(..)` written in tokens, such
/// as a `cfg_attr` argument list or a `macro_rules!` body, that names a guarded lint.
fn nested_hits(tokens: TokenStream, found: &mut Vec<Hit>) {
    let trees: Vec<TokenTree> = tokens.into_iter().collect();
    for (index, tree) in trees.iter().enumerate() {
        match (tree, trees.get(index + 1)) {
            (TokenTree::Group(group), _) => nested_hits(group.stream(), found),
            (TokenTree::Ident(level), Some(TokenTree::Group(group)))
                if LEVELS.contains(&level.to_string().as_str()) && group.delimiter() == Delimiter::Parenthesis =>
            {
                let named = paths(group.stream()).into_iter().filter(|lint| guarded(lint));
                found.extend(named.map(|lint| Hit { level: level.to_string(), lint }));
            }
            _ => {}
        }
    }
}

fn direct_hits(attribute: &syn::Attribute) -> Vec<Hit> {
    let syn::Meta::List(list) = &attribute.meta else { return Vec::new() };
    let Some(level) = LEVELS.iter().find(|level| list.path.is_ident(level)) else { return Vec::new() };
    let named = paths(list.tokens.clone()).into_iter().filter(|lint| guarded(lint));
    named.map(|lint| Hit { level: (*level).to_owned(), lint }).collect()
}

#[derive(Default)]
struct Everywhere {
    found: Vec<Hit>,
}

impl<'ast> Visit<'ast> for Everywhere {
    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        self.found.extend(direct_hits(attribute));
        if let syn::Meta::List(list) = &attribute.meta {
            nested_hits(list.tokens.clone(), &mut self.found);
        }
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        nested_hits(mac.tokens.clone(), &mut self.found);
    }
}

/// What one Rust file says about the guarded lints.
struct Facts {
    /// Module-level `allow`/`expect` of the banned methods: the boundary opt-out.
    allows: usize,
    /// Module-level `forbid`/`deny` of the banned methods.
    guards: Vec<String>,
    /// Mentions that are neither the opt-out nor the root guard.
    stray: usize,
}

fn facts(text: &str, is_root: bool) -> Result<Facts, syn::Error> {
    let file = syn::parse_file(text)?;
    let mut everywhere = Everywhere::default();
    everywhere.visit_file(&file);
    let module_level: Vec<Hit> = file.attrs.iter().flat_map(direct_hits).collect();
    let allows = module_level.iter().filter(|hit| hit.is(&["allow", "expect"])).count();
    let guards: Vec<String> =
        module_level.iter().filter(|hit| hit.is(&["forbid", "deny"])).map(|hit| hit.level.clone()).collect();
    let forbidding = guards.iter().filter(|level| *level == "forbid").count();
    let silencing = everywhere.found.iter().filter(|hit| hit.level != "deny").count();
    let accounted = allows + usize::from(is_root) * forbidding.min(1);
    Ok(Facts { allows, guards: if is_root { guards } else { Vec::new() }, stray: silencing - accounted })
}

// ---- crates ---------------------------------------------------------------------------------------

fn manifest(text: &str) -> Result<toml::Table, toml::de::Error> {
    text.parse()
}

fn declared_paths(manifest: &toml::Table) -> Vec<PathBuf> {
    let path_of = |table: &toml::Value| table.get("path").and_then(toml::Value::as_str).map(PathBuf::from);
    let library = manifest.get("lib").and_then(path_of);
    let targets = ["bin", "test", "bench", "example"]
        .into_iter()
        .filter_map(|kind| manifest.get(kind).and_then(toml::Value::as_array))
        .flatten()
        .filter_map(path_of);
    let build = manifest.get("package").and_then(|package| package.get("build")).and_then(toml::Value::as_str);
    library.into_iter().chain(targets).chain(build.map(PathBuf::from)).collect()
}

/// The crate roots of one package: cargo's conventional layout plus any declared paths.
fn package_roots(directory: &Path, manifest: &toml::Table, files: &Files) -> BTreeSet<PathBuf> {
    let conventional = files.keys().filter(|path| {
        let Ok(inside) = path.strip_prefix(directory) else { return false };
        let parts: Vec<&OsStr> = inside.iter().collect();
        let is = |names: &[&str]| {
            parts.len() == names.len() && parts.iter().zip(names).all(|(part, name)| *part == os(name))
        };
        let under = |prefix: &[&str]| {
            let inside_prefix =
                parts.len() > prefix.len() && parts.iter().zip(prefix).all(|(part, name)| *part == os(name));
            inside_prefix
                && match parts.len() - prefix.len() {
                    1 => is_rust(path),
                    2 => named(path, "main.rs"),
                    _ => false,
                }
        };
        is(&["src", "lib.rs"])
            || is(&["src", "main.rs"])
            || is(&["build.rs"])
            || ["tests", "benches", "examples"].iter().any(|folder| under(&[folder]))
            || under(&["src", "bin"])
    });
    let declared = declared_paths(manifest).into_iter().map(|path| directory.join(path));
    conventional.cloned().chain(declared).filter(|path| files.contains_key(path)).collect()
}

fn modules(items: &[syn::Item], children: &Path, files: &Files, seen: &mut BTreeSet<PathBuf>) {
    for item in items {
        let syn::Item::Mod(module) = item else { continue };
        let directory = children.join(module.ident.to_string());
        match &module.content {
            Some((_, inner)) => modules(inner, &directory, files, seen),
            None => {
                let flat = children.join(format!("{}.rs", module.ident));
                let nested = directory.join("mod.rs");
                [flat, nested].into_iter().filter(|file| files.contains_key(file)).for_each(|file| {
                    follow(&file, &directory, files, seen);
                });
            }
        }
    }
}

fn follow(file: &Path, children: &Path, files: &Files, seen: &mut BTreeSet<PathBuf>) {
    if !seen.insert(file.to_path_buf()) {
        return;
    }
    if let Ok(parsed) = syn::parse_file(&files[file]) {
        modules(&parsed.items, children, files, seen);
    }
}

/// Every file of the crate rooted at `root`, found by following its `mod` declarations.
fn crate_files(root: &Path, files: &Files) -> BTreeSet<PathBuf> {
    let mut seen = BTreeSet::new();
    follow(root, root.parent().unwrap(), files, &mut seen);
    seen
}

fn lint_overrides(path: &Path, manifest: &toml::Table) -> Vec<String> {
    let scopes = [manifest.get("lints"), manifest.get("workspace").and_then(|workspace| workspace.get("lints"))];
    let guarded = [("clippy", ["disallowed_methods", "style", "all"].as_slice()), ("rust", ["warnings"].as_slice())];
    scopes
        .into_iter()
        .flatten()
        .flat_map(|scope| {
            guarded.iter().flat_map(move |(tool, names)| {
                let table = scope.get(tool).and_then(toml::Value::as_table);
                names
                    .iter()
                    .filter(move |name| table.is_some_and(|table| table.contains_key(**name)))
                    .map(move |name| (*tool, *name))
            })
        })
        .map(|(tool, name)| format!("{}: [lints.{tool}] overrides {name}", path.display()))
        .collect()
}

// ---- the rules ----------------------------------------------------------------------------------------

fn problems(files: &Files, registry: &str) -> Vec<String> {
    let registry: Registry = match toml::from_str(registry) {
        Ok(registry) => registry,
        Err(error) => return vec![format!("{REGISTRY}: {error}")],
    };
    let mut found = Vec::new();
    let mut registered = BTreeSet::new();
    for boundary in &registry.boundary {
        let path = &boundary.path;
        let is_new = registered.insert(path.clone());
        found.extend((!is_new).then(|| format!("{}: registered twice", path.display())));
        let is_root = named(path, "lib.rs") || named(path, "main.rs") || named(path, "mod.rs");
        found.extend(is_root.then(|| format!("{}: a crate root or mod.rs cannot be a boundary", path.display())));
    }

    let mut roots = BTreeSet::new();
    for (path, text) in files.iter().filter(|(path, _)| named(path, "Cargo.toml")) {
        match manifest(text) {
            Err(error) => found.push(format!("{}: {error}", path.display())),
            Ok(table) => {
                found.extend(lint_overrides(path, &table));
                if table.contains_key("package") {
                    roots.extend(package_roots(path.parent().unwrap(), &table, files));
                }
            }
        }
    }

    for (path, text) in files.iter().filter(|(path, _)| is_rust(path)) {
        let is_root = roots.contains(path);
        let facts = match facts(text, is_root) {
            Ok(facts) => facts,
            Err(error) => {
                found.push(format!("{}: {error}", path.display()));
                continue;
            }
        };
        let shown = path.display();
        let is_registered = registered.contains(path);
        match (facts.allows, is_registered) {
            (0, true) => found.push(format!("{shown}: registered but has no module-level allow")),
            (0, false) | (1, true) => {}
            (1, false) => found.push(format!("{shown}: allow is not registered")),
            _ => found.push(format!("{shown}: more than one module-level allow")),
        }
        found.extend(
            (facts.stray > 0).then(|| {
                format!("{shown}: names a guarded lint outside its one module-level allow and the root guard")
            }),
        );
        if is_root {
            let has_boundary = crate_files(path, files).iter().any(|file| registered.contains(file));
            let (needed, why) = match has_boundary {
                true => ("deny", "the crate has a registered boundary"),
                false => ("forbid", "the crate has no registered boundary"),
            };
            found.extend((facts.guards != [needed]).then(|| {
                format!("{shown}: crate root must carry #![{needed}(clippy::disallowed_methods)] because {why}")
            }));
        }
    }
    for path in registered.iter().filter(|path| !files.contains_key(*path)) {
        found.push(format!("{}: registered but has no module-level allow", path.display()));
    }
    found.sort();
    found
}

#[test]
fn the_workspace_has_no_way_around_the_string_twiddling_ban_except_registered_boundaries() {
    let root = workspace();
    let mut files = Files::new();
    read_tree(&root, &root, &mut files);
    let registry = fs::read_to_string(root.join(REGISTRY)).unwrap();
    assert_eq!(problems(&files, &registry), Vec::<String>::new());
}

// ---- clippy.toml ------------------------------------------------------------------------------------------

#[derive(Deserialize)]
struct Clippy {
    #[serde(rename = "disallowed-methods")]
    banned: Vec<Banned>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Banned {
    path: String,
    #[allow(dead_code)]
    reason: Reason,
}

const MUST_BAN: [&str; 79] = [
    "str::replace",
    "str::replacen",
    "str::split",
    "str::split_whitespace",
    "str::split_ascii_whitespace",
    "str::splitn",
    "str::split_terminator",
    "str::split_inclusive",
    "str::split_once",
    "str::rsplit",
    "str::rsplitn",
    "str::rsplit_once",
    "str::rsplit_terminator",
    "str::split_at",
    "str::split_at_mut",
    "str::split_at_checked",
    "str::split_at_mut_checked",
    "str::trim",
    "str::trim_start",
    "str::trim_end",
    "str::trim_matches",
    "str::trim_start_matches",
    "str::trim_end_matches",
    "str::trim_left",
    "str::trim_right",
    "str::trim_ascii",
    "str::trim_ascii_start",
    "str::trim_ascii_end",
    "str::find",
    "str::rfind",
    "str::contains",
    "str::starts_with",
    "str::ends_with",
    "str::strip_prefix",
    "str::strip_suffix",
    "str::matches",
    "str::rmatches",
    "str::match_indices",
    "str::rmatch_indices",
    "str::char_indices",
    "str::chars",
    "str::lines",
    "str::get",
    "str::get_mut",
    "str::as_bytes",
    "str::bytes",
    "str::to_lowercase",
    "str::to_uppercase",
    "str::to_ascii_lowercase",
    "str::to_ascii_uppercase",
    "str::make_ascii_lowercase",
    "str::make_ascii_uppercase",
    "str::eq_ignore_ascii_case",
    "str::repeat",
    "str::escape_debug",
    "str::escape_default",
    "str::escape_unicode",
    "std::string::String::truncate",
    "std::string::String::retain",
    "std::string::String::replace_range",
    "std::string::String::insert_str",
    "std::string::String::insert",
    "std::string::String::split_off",
    "std::string::String::pop",
    "std::string::String::remove",
    "std::string::String::drain",
    "slice::join",
    "slice::concat",
    "char::escape_debug",
    "char::escape_default",
    "char::escape_unicode",
    "std::io::BufRead::lines",
    "std::io::BufRead::read_line",
    "std::io::Stdin::lines",
    "std::io::Stdin::read_line",
    "std::path::Path::to_str",
    "std::path::Path::to_string_lossy",
    "std::ffi::OsStr::to_str",
    "std::ffi::OsStr::to_string_lossy",
];

#[test]
fn clippy_toml_bans_every_string_method_family_with_a_reason_and_no_invalid_path_escape() {
    let text = fs::read_to_string(workspace().join("clippy.toml")).unwrap();
    let clippy: Clippy = toml::from_str(&text).unwrap();
    let banned: BTreeSet<&str> = clippy.banned.iter().map(|entry| entry.path.as_str()).collect();
    let missing: Vec<&str> = MUST_BAN.into_iter().filter(|path| !banned.contains(path)).collect();
    assert_eq!(missing, Vec::<&str>::new());
    assert_eq!(banned.len(), clippy.banned.len(), "a path is listed twice");
}

#[test]
fn no_regex_crate_is_locked_so_clippy_has_no_absent_regex_path_to_ban() {
    let lock: toml::Table = fs::read_to_string(workspace().join("Cargo.lock")).unwrap().parse().unwrap();
    let packages = lock["package"].as_array().unwrap();
    let names: BTreeSet<&str> = packages.iter().filter_map(|package| package["name"].as_str()).collect();
    let regexes: Vec<&str> = ["regex", "regex-lite", "fancy-regex", "onig", "pcre2"]
        .into_iter()
        .filter(|name| names.contains(name))
        .collect();
    assert_eq!(regexes, Vec::<&str>::new());
}

// ---- the rules, on small trees ----------------------------------------------------------------------------------

const PACKAGE: &str = "[package]\nname = \"a\"\nversion = \"0.1.0\"\n";
const FORBID: &str = "#![forbid(clippy::disallowed_methods)]\n";
const DENY: &str = "#![deny(clippy::disallowed_methods)]\n";
const ALLOW: &str = "#![allow(clippy::disallowed_methods)]\n";

fn tree(entries: &[(&str, &str)]) -> Files {
    entries.iter().map(|(path, text)| (PathBuf::from(path), (*text).to_owned())).collect()
}

fn registry(paths: &[&str]) -> String {
    paths
        .iter()
        .map(|path| format!("[[boundary]]\npath = \"{path}\"\nreason = \"parses ids\"\nsource = \"failure: a test\"\n"))
        .collect()
}

/// A package `a` whose root declares module `b`, with `b` written as given.
fn with_module(root: &str, module: &str) -> Files {
    tree(&[("a/Cargo.toml", PACKAGE), ("a/src/lib.rs", &format!("{root}mod b;\n")), ("a/src/b.rs", module)])
}

#[test]
fn a_root_of_a_crate_without_a_registered_boundary_must_forbid_and_one_with_a_boundary_must_deny() {
    let bare = tree(&[("a/Cargo.toml", PACKAGE), ("a/src/lib.rs", "fn f() {}")]);
    assert_eq!(
        problems(&bare, &registry(&[])),
        [
            "a/src/lib.rs: crate root must carry #![forbid(clippy::disallowed_methods)] because the crate has no registered boundary"
        ]
    );
    let boundary = format!("{DENY}mod b;");
    let denying = tree(&[("a/Cargo.toml", PACKAGE), ("a/src/lib.rs", &boundary), ("a/src/b.rs", ALLOW)]);
    assert_eq!(problems(&denying, &registry(&["a/src/b.rs"])), Vec::<String>::new());
    let forbidding = with_module(FORBID, ALLOW);
    assert_eq!(
        problems(&forbidding, &registry(&["a/src/b.rs"])),
        [
            "a/src/lib.rs: crate root must carry #![deny(clippy::disallowed_methods)] because the crate has a registered boundary"
        ]
    );
    let weakened = with_module("#![deny(clippy::disallowed_methods)]\n#![forbid(clippy::disallowed_methods)]\n", "");
    assert_eq!(
        problems(&weakened, &registry(&[])),
        [
            "a/src/lib.rs: crate root must carry #![forbid(clippy::disallowed_methods)] because the crate has no registered boundary"
        ]
    );
}

#[test]
fn every_crate_root_is_checked_binaries_tests_and_build_scripts_too_and_outside_crates() {
    let files = tree(&[
        ("x/Cargo.toml", PACKAGE),
        ("x/build.rs", "fn main() {}"),
        ("x/src/main.rs", "fn main() {}"),
        ("x/src/bin/tool.rs", FORBID),
        ("x/src/bin/many/main.rs", "fn main() {}"),
        ("x/tests/it.rs", "fn t() {}"),
        ("x/tests/shared/mod.rs", "fn shared() {}"),
        ("x/examples/e.rs", FORBID),
        ("xtask/Cargo.toml", "[package]\nname = \"xtask\"\n\n[[bin]]\nname = \"xtask\"\npath = \"run.rs\"\n"),
        ("xtask/run.rs", "fn main() {}"),
    ]);
    let guardless = |path: &str| {
        format!(
            "{path}: crate root must carry #![forbid(clippy::disallowed_methods)] because the crate has no registered boundary"
        )
    };
    assert_eq!(
        problems(&files, &registry(&[])),
        [
            guardless("x/build.rs"),
            guardless("x/src/bin/many/main.rs"),
            guardless("x/src/main.rs"),
            guardless("x/tests/it.rs"),
            guardless("xtask/run.rs"),
        ]
    );
}

#[test]
fn a_boundary_in_a_nested_module_makes_its_whole_crate_a_boundary_crate_but_not_its_sibling_crates() {
    let files = tree(&[
        ("a/Cargo.toml", PACKAGE),
        ("a/src/lib.rs", "#![deny(clippy::disallowed_methods)]\nmod outer { mod inner; }\n"),
        ("a/src/outer/inner.rs", ALLOW),
        ("a/tests/t.rs", FORBID),
    ]);
    assert_eq!(problems(&files, &registry(&["a/src/outer/inner.rs"])), Vec::<String>::new());
}

#[test]
fn every_way_of_naming_a_guarded_lint_outside_the_one_module_level_allow_is_reported() {
    let bypasses = [
        "#[allow(clippy::disallowed_methods)] fn f() {}",
        "#[allow(clippy::all)] fn f() {}",
        "#[expect(clippy::style)] fn f() {}",
        "#[warn(clippy)] fn f() {}",
        "#[allow(warnings)] fn f() {}",
        "#[forbid(clippy::all)] fn f() {}",
        "#![allow(clippy::style)]",
        "mod m { #![expect(clippy::disallowed_methods)] }",
        "#[cfg_attr(test, allow(clippy::disallowed_methods))] fn f() {}",
        "#[cfg_attr(all(unix, test), expect(clippy::all, reason = \"x\"))] fn f() {}",
        "macro_rules! m { () => { #[allow(clippy::style)] fn f() {} }; }",
        "macro_rules! m { () => { #![warn(warnings)] }; }",
        "macro_rules! m { () => { #[cfg_attr(test, forbid(clippy))] fn f() {} }; }",
        "m! { #[allow(clippy::disallowed_methods)] fn f() {} }",
        "#![forbid(clippy::disallowed_methods)]",
        "#![warn(clippy::disallowed_methods)]",
    ];
    let stray = "a/src/b.rs: names a guarded lint outside its one module-level allow and the root guard";
    bypasses.into_iter().for_each(|code| {
        let files = with_module(FORBID, code);
        assert_eq!(problems(&files, &registry(&[])), [stray], "{code}");
    });
}

#[test]
fn lint_attributes_that_do_not_touch_a_guarded_lint_pass() {
    let fine = [
        "#[allow(dead_code)] fn f() {}",
        "#![allow(clippy::too_many_arguments)]",
        "#[cfg_attr(test, allow(dead_code))] fn f() {}",
        "#[deny(clippy::all)] fn f() {}",
        "macro_rules! m { () => { #[allow(unused)] fn f() {} }; }",
        "#[derive(Debug)] struct S;",
    ];
    fine.into_iter().for_each(|code| {
        assert_eq!(problems(&with_module(FORBID, code), &registry(&[])), Vec::<String>::new(), "{code}");
    });
}

#[test]
fn a_module_level_allow_must_be_registered_exactly_once_per_file_and_warn_is_no_boundary() {
    let allowed = with_module(DENY, ALLOW);
    assert_eq!(
        problems(&allowed, &registry(&[])),
        [
            "a/src/b.rs: allow is not registered",
            "a/src/lib.rs: crate root must carry #![forbid(clippy::disallowed_methods)] because the crate has no registered boundary",
        ]
    );
    assert_eq!(problems(&allowed, &registry(&["a/src/b.rs"])), Vec::<String>::new());
    let twice = with_module(DENY, &format!("{ALLOW}#![expect(clippy::disallowed_methods)]\n"));
    assert_eq!(problems(&twice, &registry(&["a/src/b.rs"])), ["a/src/b.rs: more than one module-level allow"]);
    let warned = with_module(FORBID, "#![warn(clippy::disallowed_methods)]\n");
    assert_eq!(
        problems(&warned, &registry(&["a/src/b.rs"])),
        [
            "a/src/b.rs: names a guarded lint outside its one module-level allow and the root guard",
            "a/src/b.rs: registered but has no module-level allow",
            "a/src/lib.rs: crate root must carry #![deny(clippy::disallowed_methods)] because the crate has a registered boundary",
        ]
    );
}

#[test]
fn a_registered_boundary_whose_allow_is_gone_or_file_is_missing_is_reported() {
    let files = with_module(FORBID, "fn f() {}");
    assert_eq!(
        problems(&files, &registry(&["a/src/b.rs", "gone.rs"])),
        [
            "a/src/b.rs: registered but has no module-level allow",
            "a/src/lib.rs: crate root must carry #![deny(clippy::disallowed_methods)] because the crate has a registered boundary",
            "gone.rs: registered but has no module-level allow",
        ]
    );
}

#[test]
fn a_crate_root_or_mod_rs_cannot_be_registered() {
    let files = tree(&[
        ("a/Cargo.toml", PACKAGE),
        ("a/src/lib.rs", &format!("{DENY}{ALLOW}")),
        ("a/src/main.rs", &format!("{DENY}{ALLOW}")),
        ("a/src/m/mod.rs", ALLOW),
    ]);
    let paths = ["a/src/lib.rs", "a/src/main.rs", "a/src/m/mod.rs"];
    let refused = paths.map(|path| format!("{path}: a crate root or mod.rs cannot be a boundary"));
    let reported = problems(&files, &registry(&paths));
    assert_eq!(refused.iter().filter(|line| reported.contains(line)).count(), 3, "{reported:?}");
}

#[test]
fn a_manifest_lint_override_of_a_guarded_lint_is_reported_in_any_cargo_toml() {
    let cases = [
        ("[lints.clippy]\ndisallowed_methods = \"allow\"\n", "[lints.clippy] overrides disallowed_methods"),
        ("[lints.clippy]\nstyle = { level = \"allow\", priority = 1 }\n", "[lints.clippy] overrides style"),
        ("[workspace.lints.clippy]\nall = \"allow\"\n", "[lints.clippy] overrides all"),
        ("[lints.rust]\nwarnings = \"allow\"\n", "[lints.rust] overrides warnings"),
        ("[workspace.lints.rust]\nwarnings = \"allow\"\n", "[lints.rust] overrides warnings"),
    ];
    cases.into_iter().for_each(|(table, why)| {
        let manifest = format!("{PACKAGE}{table}");
        let files = tree(&[("deep/er/Cargo.toml", &manifest), ("deep/er/src/lib.rs", FORBID)]);
        assert_eq!(problems(&files, &registry(&[])), [format!("deep/er/Cargo.toml: {why}")]);
    });
    let fine = format!("{PACKAGE}[lints]\nworkspace = true\n\n[lints.rust]\nunsafe_code = \"forbid\"\n");
    let files = tree(&[("a/Cargo.toml", &fine), ("a/src/lib.rs", FORBID)]);
    assert_eq!(problems(&files, &registry(&[])), Vec::<String>::new());
}

#[test]
fn registry_entries_need_a_reason_a_dated_source_and_known_fields_and_a_path_only_once() {
    let files = with_module(DENY, ALLOW);
    let entry = |reason: &str, source: &str, extra: &str| {
        format!("[[boundary]]\npath = \"a/src/b.rs\"\nreason = \"{reason}\"\nsource = \"{source}\"\n{extra}")
    };
    let rejected = [
        entry("", "failure: x", ""),
        entry("  ", "failure: x", ""),
        entry("r", "because", ""),
        entry("r", "owner 2026-10-30:", ""),
        entry("r", "failure: x", "note = \"y\"\n"),
    ];
    rejected.into_iter().for_each(|text| {
        assert!(toml::from_str::<Registry>(&text).is_err(), "{text}");
        assert_eq!(problems(&files, &text).len(), 1, "{text}");
    });
    let fine = entry("parses ids", "owner 2026-10-30: text is parsed once", "");
    assert_eq!(problems(&files, &fine), Vec::<String>::new());
    let twice = format!("{fine}{fine}");
    assert_eq!(problems(&files, &twice), ["a/src/b.rs: registered twice"]);
}

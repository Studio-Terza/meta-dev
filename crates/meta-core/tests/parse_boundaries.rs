//! The string-twiddling ban (`clippy.toml`, owner 2026-10-30) is a coarse speed bump, set once for
//! the workspace. This test keeps its one way out visible. Three rules:
//! 1. every crate manifest has `lints.workspace = true`;
//! 2. `allow(clippy::disallowed_methods)` appears only in files registered in
//!    `self/parse-boundaries.toml`, once per file;
//! 3. no manifest overrides the lint: the workspace sets it to `deny` and no crate sets it.

use meta_core::Reason;
use meta_testkit::Scratch;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

type Tree = BTreeMap<PathBuf, String>;

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

/// Where a boundary came from: an owner decision on a date, or a failure.
#[derive(Deserialize)]
#[serde(untagged, deny_unknown_fields)]
enum Source {
    Owner {
        #[allow(dead_code)]
        owner: Day,
        #[allow(dead_code)]
        words: Reason,
    },
    Failure {
        #[allow(dead_code)]
        failure: Reason,
    },
}

/// A native TOML date, such as `2026-10-30`: no time, no offset.
#[derive(Deserialize)]
#[serde(try_from = "toml::value::Datetime")]
struct Day;

impl TryFrom<toml::value::Datetime> for Day {
    type Error = &'static str;

    fn try_from(stamp: toml::value::Datetime) -> Result<Self, &'static str> {
        match (stamp.date, stamp.time, stamp.offset) {
            (Some(_), None, None) => Ok(Self),
            _ => Err("an owner source takes a date only, such as 2026-10-30"),
        }
    }
}

fn read_tree(directory: &Path, root: &Path, tree: &mut Tree) {
    let mut entries: Vec<_> = fs::read_dir(directory).unwrap().map(Result::unwrap).collect();
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let kind = entry.file_type().unwrap();
        let name = entry.file_name();
        match () {
            () if kind.is_dir() && name != "target" && name != ".git" => read_tree(&path, root, tree),
            () if kind.is_file() && (path.extension().is_some_and(|ext| ext == "rs") || name == "Cargo.toml") => {
                tree.insert(path.strip_prefix(root).unwrap().to_path_buf(), fs::read_to_string(&path).unwrap());
            }
            () => {}
        }
    }
}

/// The `allow(clippy::disallowed_methods)` attributes in one Rust file, anywhere in it.
#[derive(Default)]
struct Allows(usize);

impl<'ast> Visit<'ast> for Allows {
    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if !(attribute.path().is_ident("allow") || attribute.path().is_ident("expect")) {
            return;
        }
        let _ = attribute.parse_nested_meta(|meta| {
            let path: Vec<String> = meta.path.segments.iter().map(|segment| segment.ident.to_string()).collect();
            match path.as_slice() {
                [tool, lint] if tool == "clippy" && lint == "disallowed_methods" => self.0 += 1,
                _ => {}
            }
            match meta.input.peek(syn::Token![=]) {
                true => meta.value()?.parse::<syn::Expr>().map(drop),
                false => Ok(()),
            }
        });
    }
}

fn allows(text: &str) -> usize {
    let mut found = Allows::default();
    found.visit_file(&syn::parse_file(text).unwrap());
    found.0
}

fn clippy_lint(manifest: &toml::Table, table: &str) -> Option<toml::Value> {
    let lints = match table {
        "workspace" => manifest.get("workspace")?.get("lints")?,
        _ => manifest.get("lints")?,
    };
    lints.get("clippy")?.get("disallowed_methods").cloned()
}

fn violations(tree: &Tree, registry: &Registry) -> Vec<String> {
    let mut found = Vec::new();
    for (path, text) in tree {
        let shown = path.display();
        match path.file_name().is_some_and(|name| name == "Cargo.toml") {
            true => {
                let manifest: toml::Table = text.parse().unwrap();
                let inherits = manifest.get("lints").and_then(|lints| lints.get("workspace")) == Some(&true.into());
                if manifest.contains_key("package") && !inherits {
                    found.push(format!("{shown}: no `lints.workspace = true`"));
                }
                if clippy_lint(&manifest, "crate").is_some() {
                    found.push(format!("{shown}: sets clippy::disallowed_methods itself"));
                }
                if manifest.contains_key("workspace") && clippy_lint(&manifest, "workspace") != Some("deny".into()) {
                    found.push(format!("{shown}: workspace does not set clippy::disallowed_methods = \"deny\""));
                }
            }
            false => {
                let count = allows(text);
                let registered = registry.boundary.iter().any(|boundary| boundary.path == *path);
                if count > 0 && !registered {
                    found.push(format!("{shown}: allows clippy::disallowed_methods but is not registered"));
                }
                if count > 1 {
                    found.push(format!("{shown}: allows clippy::disallowed_methods {count} times"));
                }
            }
        }
    }
    found
}

fn registry(text: &str) -> Registry {
    toml::from_str(text).unwrap()
}

fn tree(files: &[(&str, &str)]) -> Tree {
    files.iter().map(|(path, text)| (PathBuf::from(path), (*text).to_owned())).collect()
}

const GOOD_MANIFEST: &str = "[package]\nname = \"a\"\n[lints]\nworkspace = true\n";
const ROOT: &str = "[workspace]\nmembers = []\n[workspace.lints.clippy]\ndisallowed_methods = \"deny\"\n";
const REGISTERED: &str = "[[boundary]]\npath = \"a/src/p.rs\"\nreason = \"r\"\nsource = { failure = \"f\" }\n";
const ALLOW: &str = "#![allow(clippy::disallowed_methods)]\nfn f() {}\n";

#[test]
fn the_workspace_keeps_the_three_rules() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Tree::new();
    read_tree(&root, &root, &mut files);
    let registry = registry(&fs::read_to_string(root.join("self/parse-boundaries.toml")).unwrap());
    assert_eq!(violations(&files, &registry), Vec::<String>::new());
}

#[test]
fn an_allow_needs_a_registered_file_and_appears_once_in_it() {
    let files = tree(&[("a/src/p.rs", ALLOW), ("a/src/q.rs", ALLOW), ("a/src/r.rs", "fn f() {}\n")]);
    assert_eq!(
        violations(&files, &registry(REGISTERED)),
        ["a/src/q.rs: allows clippy::disallowed_methods but is not registered"]
    );
    let twice =
        "#![allow(clippy::disallowed_methods)]\n#[expect(clippy::disallowed_methods, reason = \"x\")]\nfn f() {}\n";
    assert_eq!(
        violations(&tree(&[("a/src/p.rs", twice)]), &registry(REGISTERED)),
        ["a/src/p.rs: allows clippy::disallowed_methods 2 times"]
    );
    let other = "#![allow(clippy::needless_return, dead_code)]\nfn f() {}\n";
    assert_eq!(violations(&tree(&[("a/src/q.rs", other)]), &registry(REGISTERED)), Vec::<String>::new());
}

#[test]
fn a_crate_inherits_the_workspace_lints_and_no_manifest_overrides_the_ban() {
    let own = "[package]\nname = \"a\"\n[lints.clippy]\ndisallowed_methods = \"allow\"\n";
    let weak = "[workspace]\n[workspace.lints.clippy]\ndisallowed_methods = \"warn\"\n";
    let files =
        tree(&[("Cargo.toml", weak), ("a/Cargo.toml", own), ("b/Cargo.toml", GOOD_MANIFEST), ("c/Cargo.toml", ROOT)]);
    assert_eq!(
        violations(&files, &registry("")),
        [
            "Cargo.toml: workspace does not set clippy::disallowed_methods = \"deny\"",
            "a/Cargo.toml: no `lints.workspace = true`",
            "a/Cargo.toml: sets clippy::disallowed_methods itself",
        ]
    );
}

#[test]
fn the_tree_reader_follows_no_symlinks() {
    let scratch = Scratch::new("tree");
    let root = scratch.join("tree");
    let outside = scratch.join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("real.rs"), "fn f() {}\n").unwrap();
    fs::write(outside.join("far.rs"), ALLOW).unwrap();
    std::os::unix::fs::symlink(root.join("real.rs"), root.join("file_link.rs")).unwrap();
    std::os::unix::fs::symlink(&outside, root.join("dir_link")).unwrap();
    std::os::unix::fs::symlink(&root, root.join("loop")).unwrap();
    let mut files = Tree::new();
    read_tree(&root, &root, &mut files);
    assert_eq!(files.keys().collect::<Vec<_>>(), [Path::new("real.rs")]);
}

#[test]
fn a_registry_source_is_an_owner_date_with_words_or_a_failure() {
    let entry = |source: &str| {
        toml::from_str::<Registry>(&format!("[[boundary]]\npath = \"p\"\nreason = \"r\"\nsource = {source}\n")).is_ok()
    };
    let parsed = [
        "{ owner = 2026-10-30, words = \"w\" }",
        "{ failure = \"f\" }",
        "{ owner = \"2026-10-30\", words = \"w\" }",
        "{ owner = 2026-10-30T10:00:00, words = \"w\" }",
        "{ owner = 2026-10-30 }",
        "{ owner = 2026-10-30, words = \"\" }",
        "{ failure = \"\" }",
        "{ failure = \"f\", words = \"w\" }",
        "\"owner 2026-10-30: w\"",
    ]
    .map(entry);
    assert_eq!(parsed, [true, true, false, false, false, false, false, false, false]);
}

//! Every `#![allow(clippy::disallowed_methods)]` must be a registered parse boundary in
//! `self/parse-boundaries.toml`, and every registered boundary must still carry its allow.
//! Source is read with `syn` and compared as paths and components, never as text.

use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

#[derive(Deserialize)]
struct Registry {
    boundary: Vec<Boundary>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Boundary {
    path: PathBuf,
    reason: String,
    source: String,
}

struct Allows {
    found: usize,
}

impl<'ast> Visit<'ast> for Allows {
    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        self.found += usize::from(allows_disallowed_methods(attribute));
    }
}

fn allows_disallowed_methods(attribute: &syn::Attribute) -> bool {
    let silencing = ["allow", "expect", "warn"].iter().any(|level| attribute.path().is_ident(level));
    let mut hit = false;
    if silencing {
        let _ = attribute.parse_nested_meta(|lint| {
            let names: Vec<_> = lint.path.segments.iter().map(|segment| segment.ident.to_string()).collect();
            hit |= names == ["clippy", "disallowed_methods"];
            Ok(())
        });
    }
    hit
}

/// Files whose module-level (inner, file-root) attributes carry the allow, and files with the
/// allow anywhere else, which is never permitted.
struct Scan {
    boundaries: BTreeSet<PathBuf>,
    misplaced: BTreeSet<PathBuf>,
}

fn scan(sources: &[(PathBuf, String)]) -> Scan {
    let mut boundaries = BTreeSet::new();
    let mut misplaced = BTreeSet::new();
    for (path, source) in sources {
        let file = syn::parse_file(source).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let module_level = file.attrs.iter().filter(|attribute| allows_disallowed_methods(attribute)).count();
        let mut everywhere = Allows { found: 0 };
        everywhere.visit_file(&file);
        match (module_level, everywhere.found) {
            (0, 0) => {}
            (module, all) if module == all => {
                boundaries.insert(path.clone());
            }
            _ => {
                misplaced.insert(path.clone());
            }
        }
    }
    Scan { boundaries, misplaced }
}

fn problems(sources: &[(PathBuf, String)], registered: &BTreeSet<PathBuf>) -> Vec<String> {
    let scanned = scan(sources);
    let unregistered =
        scanned.boundaries.difference(registered).map(|path| format!("{}: allow is not registered", path.display()));
    let stale = registered
        .difference(&scanned.boundaries)
        .map(|path| format!("{}: registered but has no module-level allow", path.display()));
    let misplaced = scanned
        .misplaced
        .iter()
        .map(|path| format!("{}: allow must be module-level, once, at the file root", path.display()));
    unregistered.chain(stale).chain(misplaced).collect()
}

fn rust_files(directory: &Path, found: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = fs::read_dir(directory).unwrap().map(|entry| entry.unwrap().path()).collect();
    entries.sort();
    for path in entries {
        match path.is_dir() {
            true => rust_files(&path, found),
            false => found.extend(Some(path).filter(|path| path.extension().is_some_and(|ext| ext == "rs"))),
        }
    }
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn every_allow_is_a_registered_boundary_and_every_boundary_still_exists() {
    let root = workspace();
    let registry: Registry =
        toml::from_str(&fs::read_to_string(root.join("self/parse-boundaries.toml")).unwrap()).unwrap();
    let registered: BTreeSet<PathBuf> = registry.boundary.into_iter().map(|boundary| boundary.path).collect();
    let mut files = Vec::new();
    rust_files(&root.join("crates"), &mut files);
    let sources: Vec<(PathBuf, String)> = files
        .into_iter()
        .map(|file| (file.strip_prefix(&root).unwrap().to_path_buf(), fs::read_to_string(&file).unwrap()))
        .collect();
    assert_eq!(problems(&sources, &registered), Vec::<String>::new());
}

fn source(path: &str, text: &str) -> (PathBuf, String) {
    (PathBuf::from(path), text.to_owned())
}

#[test]
fn an_unregistered_module_level_allow_is_reported() {
    let sources = [source("a.rs", "#![allow(clippy::disallowed_methods)]\nfn f() {}")];
    assert_eq!(problems(&sources, &BTreeSet::new()), ["a.rs: allow is not registered"]);
}

#[test]
fn a_registered_boundary_whose_allow_is_gone_or_file_is_missing_is_reported() {
    let registered = BTreeSet::from([PathBuf::from("a.rs"), PathBuf::from("gone.rs")]);
    let sources = [source("a.rs", "fn f() {}")];
    assert_eq!(
        problems(&sources, &registered),
        ["a.rs: registered but has no module-level allow", "gone.rs: registered but has no module-level allow"]
    );
}

#[test]
fn an_item_level_or_inline_module_allow_is_misplaced_even_when_registered() {
    let registered = BTreeSet::from([PathBuf::from("a.rs"), PathBuf::from("b.rs")]);
    let sources = [
        source("a.rs", "#[allow(clippy::disallowed_methods)]\nfn f() {}"),
        source("b.rs", "mod m { #![expect(clippy::disallowed_methods)] }"),
    ];
    assert_eq!(
        problems(&sources, &registered),
        [
            "a.rs: registered but has no module-level allow",
            "b.rs: registered but has no module-level allow",
            "a.rs: allow must be module-level, once, at the file root",
            "b.rs: allow must be module-level, once, at the file root"
        ]
    );
}

#[test]
fn a_registered_module_level_allow_passes_and_other_lints_are_ignored() {
    let registered = BTreeSet::from([PathBuf::from("a.rs")]);
    let sources = [
        source("a.rs", "#![allow(clippy::disallowed_methods)]\nfn f() {}"),
        source("b.rs", "#![allow(clippy::too_many_arguments)]\n#[allow(dead_code)]\nfn g() {}"),
    ];
    assert_eq!(problems(&sources, &registered), Vec::<String>::new());
}

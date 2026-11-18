use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, Memory};
use lint_layout::Manifest;

const TARGETS: &str =
    "[package]\nname = \"p\"\n\n[lib]\npath = \"src/lib.rs\"\n\n[[bin]]\nname = \"tool\"\npath = \"src/main.rs\"\n";

#[test]
fn a_manifest_declares_its_library_and_listed_targets_as_roots() {
    let files: Memory = [(PathBuf::from("/p/Cargo.toml"), Bytes::from(TARGETS))].into_iter().collect();
    let read = Manifest::read_many(&files, &[Path::new("/p")]);
    let roots: Vec<BTreeSet<PathBuf>> = read
        .iter()
        .flatten()
        .flatten()
        .map(|manifest| manifest.declared_roots(|path| Some(path.to_path_buf())))
        .collect();
    assert_eq!(roots, [BTreeSet::from([PathBuf::from("/p/src/lib.rs"), PathBuf::from("/p/src/main.rs")])]);
}

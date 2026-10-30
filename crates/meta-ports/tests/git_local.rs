use meta_core::SelfFile;
use meta_ports::PortError;
use meta_ports::real::GitCli;
use std::fs;
use std::path::{Path, PathBuf};

const TREE: &str = "[[kind]]\nid = \"tree\"\ntext = \"t\"\nsource = \"s\"\nnames_merged_pr = false\n";
const CACHED: &str = "[[kind]]\nid = \"cached\"\ntext = \"t\"\nsource = \"s\"\nnames_merged_pr = false\n";
const GARBLED: &str = "this is [not toml";

fn scratch(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{nanos}"));
    fs::create_dir_all(root.join("tree/self")).unwrap();
    fs::create_dir_all(root.join("cache/self")).unwrap();
    root
}

fn local(root: &Path) -> Result<String, PortError> {
    GitCli::local_from(SelfFile::EscapeKinds, &root.join("tree"), Ok(root.join("cache")))
}

#[test]
fn local_falls_back_to_the_cache_when_the_working_tree_escapes_exist_but_do_not_parse() {
    let root = scratch("garbled");
    fs::write(root.join("tree/self/escapes.toml"), GARBLED).unwrap();
    fs::write(root.join("cache/self/escapes.toml"), CACHED).unwrap();
    assert_eq!(local(&root).as_deref(), Ok(CACHED));
}

#[test]
fn local_prefers_a_working_tree_file_that_parses_and_uses_the_cache_when_there_is_none() {
    let root = scratch("parsed");
    fs::write(root.join("cache/self/escapes.toml"), CACHED).unwrap();
    assert_eq!(local(&root).as_deref(), Ok(CACHED));
    fs::write(root.join("tree/self/escapes.toml"), TREE).unwrap();
    assert_eq!(local(&root).as_deref(), Ok(TREE));
}

#[test]
fn local_shows_the_working_tree_text_when_neither_copy_parses_so_the_caller_reports_why() {
    let root = scratch("neither");
    fs::write(root.join("tree/self/escapes.toml"), GARBLED).unwrap();
    assert_eq!(local(&root).as_deref(), Ok(GARBLED));
    fs::write(root.join("cache/self/escapes.toml"), GARBLED).unwrap();
    assert_eq!(local(&root).as_deref(), Ok(GARBLED));
}

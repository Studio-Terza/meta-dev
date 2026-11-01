use std::error::Error;
use std::fs;

use lint_core::Lint;
use lint_process_starters::{CommandPaths, ProcessStarters, Scope};

use lint_fixtures::{Case, Source, Sources};

const IMPORTED: &str = "use std::process::Command;\nfn f() {\n    Command::new(\"ls\").status();\n}\n";
const OTHER_COMMAND: &str = "use clap::Command;\nfn f() {\n    Command::new(\"ls\").status();\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    use std::process::Command;\n    fn f() {\n        Command::new(\"ls\").status();\n    }\n}\n";

const ALIASED: &str = "use std::process as p;\nfn f() {\n    p::Command::new(\"ls\").status();\n}\n";

const INLINE_PORTS: &str =
    "mod ports {\n    use std::process::Command;\n    pub fn f() {\n        Command::new(\"ls\").status();\n    }\n}\n";
const INLINE_OTHER: &str = "mod adapters {\n    use std::process::Command;\n    pub fn f() {\n        Command::new(\"ls\").status();\n    }\n}\n";

const IN_MACRO: &str = "fn f() {\n    println!(\"{:?}\", std::process::Command::new(\"ls\").status());\n}\n";
const SPLIT_CHAIN: &str =
    "use std::process::Command;\nfn f() {\n    Command::new(\"ls\")\n        .arg(\"x\")\n        .status();\n}\n";
const CONFIGURED_ONLY: &str =
    "use std::process::Command;\nfn f() {\n    Command::new(\"ls\")\n        .arg(\"x\");\n}\n";
const FOREIGN_RENAME: &str = "use clap::Command as Run;\nfn f() {\n    Run::new(\"ls\").status();\n}\n";
const OTHER_RENAME: &str = "use std::process::Stdio as Run;\nfn f() {\n    Run::new(\"ls\").status();\n}\n";
const GLOB: &str = "use std::process::*;\nfn f() {\n    Command::new(\"ls\").status();\n}\n";
const FOREIGN_GLOB: &str = "use clap::*;\nfn f() {\n    Command::new(\"ls\").status();\n}\n";
const THREE_SEGMENTS: &str = "fn f() {\n    a::b::c(\"ls\").status();\n}\n";
const OTHER_TYPE: &str = "fn f() {\n    process::Other::new(\"ls\").status();\n}\n";
const NEW_OF_ANOTHER: &str = "fn f() {\n    a::b::new(\"ls\").status();\n}\n";
const OTHER_CONSTRUCTOR: &str = "fn f() {\n    process::Command::other(\"ls\").status();\n}\n";

const CASES: [Case; 20] = [
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &["a.rs:5"]),
    Case::new("integration test", &[Source::new("tests/spawns.rs", IMPORTED)], &["tests/spawns.rs:3"]),
    Case::new("module alias", &[Source::new("a.rs", ALIASED)], &["a.rs:3"]),
    Case::new("ports module file", &[Source::new("crates/x/src/ports/run.rs", IMPORTED)], &[]),
    Case::new("ports module root", &[Source::new("crates/x/src/ports.rs", IMPORTED)], &[]),
    Case::new("inline ports module", &[Source::new("a.rs", INLINE_PORTS)], &[]),
    Case::new("inline other module", &[Source::new("a.rs", INLINE_OTHER)], &["a.rs:4"]),
    Case::new(
        "name only resembling ports",
        &[Source::new("crates/x/src/support/run.rs", IMPORTED)],
        &["crates/x/src/support/run.rs:3"],
    ),
    Case::new(
        "ports in a test target",
        &[Source::new("crates/x/tests/ports.rs", IMPORTED)],
        &["crates/x/tests/ports.rs:3"],
    ),
    Case::new("inside a macro argument", &[Source::new("a.rs", IN_MACRO)], &["a.rs:2"]),
    Case::new("a chain over lines", &[Source::new("a.rs", SPLIT_CHAIN)], &["a.rs:5"]),
    Case::new("a chain with no start", &[Source::new("a.rs", CONFIGURED_ONLY)], &["a.rs:3"]),
    Case::new("a foreign renamed Command", &[Source::new("a.rs", FOREIGN_RENAME)], &[]),
    Case::new("another renamed process type", &[Source::new("a.rs", OTHER_RENAME)], &[]),
    Case::new("a process glob", &[Source::new("a.rs", GLOB)], &["a.rs:3"]),
    Case::new("a foreign glob", &[Source::new("a.rs", FOREIGN_GLOB)], &[]),
    Case::new("three segments", &[Source::new("a.rs", THREE_SEGMENTS)], &[]),
    Case::new("another process type", &[Source::new("a.rs", OTHER_TYPE)], &[]),
    Case::new("new of another path", &[Source::new("a.rs", NEW_OF_ANOTHER)], &[]),
    Case::new("another Command constructor", &[Source::new("a.rs", OTHER_CONSTRUCTOR)], &[]),
];

const PACKAGES: [(&str, usize); 4] = [("meta-ports", 0), ("ports", 0), ("meta-support", 1), ("meta-run", 1)];

#[test]
fn starting_a_process_is_flagged() {
    assert_eq!(Sources::mismatches::<ProcessStarters>(&CASES), Vec::<String>::new());
}

#[test]
fn a_crate_whose_name_ends_in_ports_may_start_processes() -> Result<(), Box<dyn Error>> {
    let mut wrong = Vec::new();
    for (name, flagged) in PACKAGES {
        let root = tempfile::tempdir()?;
        fs::create_dir(root.path().join("src"))?;
        fs::write(root.path().join("Cargo.toml"), format!("[package]\nname = \"{name}\"\n"))?;
        let file = Sources::file(root.path().join("src/lib.rs"), IMPORTED);
        let found = ProcessStarters::report(&[file]).violations().len();
        if found != flagged {
            wrong.push(format!("{name} flagged {found}"));
        }
    }
    assert_eq!(wrong, Vec::<String>::new());
    Ok(())
}

#[test]
fn every_start_is_counted_including_those_behind_ports() {
    let ported = Sources::file("crates/x/src/ports/run.rs", IMPORTED);
    let inline = Sources::file("a.rs", INLINE_PORTS);
    let other = Sources::file("b.rs", OTHER_COMMAND);
    let counts: Vec<usize> =
        [ported, inline, other].iter().map(|file| CommandPaths::found(file, Scope::Everywhere).len()).collect();
    assert_eq!(counts, vec![1, 1, 0]);
}

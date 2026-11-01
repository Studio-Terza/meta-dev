use std::error::Error;
use std::fs;

use lint_core::Lint;
use lint_process_starters::ProcessStarters;

use lint_fixtures::{Case, Source, Sources};

const IMPORTED: &str = "use std::process::Command;\nfn f() {\n    Command::new(\"ls\").status();\n}\n";
const QUALIFIED: &str = "fn f() {\n    std::process::Command::new(\"ls\").spawn();\n}\n";
const RENAMED: &str = "use std::process::Command as Run;\nfn f() {\n    Run::new(\"ls\").output();\n}\n";
const BUILT_ONLY: &str = "use std::process::Command;\nfn f() -> Command {\n    Command::new(\"ls\")\n}\n";
const OTHER_COMMAND: &str = "use clap::Command;\nfn f() {\n    Command::new(\"ls\").status();\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    use std::process::Command;\n    fn f() {\n        Command::new(\"ls\").status();\n    }\n}\n";

const ALIASED: &str = "use std::process as p;\nfn f() {\n    p::Command::new(\"ls\").status();\n}\n";

const INLINE_PORTS: &str =
    "mod ports {\n    use std::process::Command;\n    pub fn f() {\n        Command::new(\"ls\").status();\n    }\n}\n";
const INLINE_OTHER: &str = "mod adapters {\n    use std::process::Command;\n    pub fn f() {\n        Command::new(\"ls\").status();\n    }\n}\n";

const CASES: [Case; 14] = [
    Case::new("imported", &[Source::new("a.rs", IMPORTED)], &["a.rs:3"]),
    Case::new("qualified", &[Source::new("a.rs", QUALIFIED)], &["a.rs:2"]),
    Case::new("renamed", &[Source::new("a.rs", RENAMED)], &["a.rs:3"]),
    Case::new("built only", &[Source::new("a.rs", BUILT_ONLY)], &["a.rs:3"]),
    Case::new("other command", &[Source::new("a.rs", OTHER_COMMAND)], &[]),
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

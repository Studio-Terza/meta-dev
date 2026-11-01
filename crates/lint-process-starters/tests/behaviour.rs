use std::error::Error;
use std::fs;

use lint_core::Lint;
use lint_process_starters::{CommandPaths, ProcessStarters, Scope};

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

const USE_AFTER: &str = "fn f() {\n    Command::new(\"ls\").status();\n}\nuse std::process::Command;\n";
const SELF_RENAMED: &str = "use std::process::{self as p};\nfn f() {\n    p::Command::new(\"ls\").status();\n}\n";
const FUNCTION_VALUE: &str = "fn f() {\n    let make = std::process::Command::new;\n    make(\"ls\").status();\n}\n";
const TYPE_ALIAS: &str = "type C = std::process::Command;\nfn f() {\n    C::new(\"ls\").status();\n}\n";
const ROOT_EXPORT: &str = "pub use std::process::Command;\nmod user;\n";
const RE_EXPORTED: &str = "pub fn f() {\n    crate::Command::new(\"ls\").status();\n}\n";
const QUALIFIED_SELF: &str = "fn f() {\n    <std::process::Command>::new(\"ls\").status();\n}\n";
const MACRO_GUARD: &str =
    "fn f() {\n    matches!(std::process::Command::new(\"ls\").status(), Ok(s) if s.success());\n}\n";
const MACRO_RULES: &str =
    "macro_rules! go {\n    () => {\n        std::process::Command::new(\"ls\").status()\n    };\n}\n";
const REPEAT_MACRO: &str = "fn f() {\n    vec![std::process::Command::new(\"ls\").status().is_ok(); 2];\n}\n";
const THROUGH_PORTS: &str =
    "mod ports {\n    pub use std::process::Command;\n}\npub fn f() {\n    ports::Command::new(\"ls\").status();\n}\n";
const SIBLING_IMPORT: &str = "mod a {\n    pub use std::process::Command;\n}\nmod b {\n    pub struct Command;\n    impl Command {\n        pub fn new(_x: u8) {}\n    }\n    pub fn f() {\n        Command::new(1);\n    }\n}\n";

const CASES: [Case; 25] = [
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
    Case::new("use after the code", &[Source::new("a.rs", USE_AFTER)], &["a.rs:2"]),
    Case::new("self renamed", &[Source::new("a.rs", SELF_RENAMED)], &["a.rs:3"]),
    Case::new("function value", &[Source::new("a.rs", FUNCTION_VALUE)], &["a.rs:2"]),
    Case::new("type alias", &[Source::new("a.rs", TYPE_ALIAS)], &["a.rs:3"]),
    Case::new(
        "re-export in the crate root",
        &[Source::new("crates/x/src/lib.rs", ROOT_EXPORT), Source::new("crates/x/src/user.rs", RE_EXPORTED)],
        &["crates/x/src/user.rs:2"],
    ),
    Case::new("qualified self", &[Source::new("a.rs", QUALIFIED_SELF)], &["a.rs:2"]),
    Case::new("macro guard", &[Source::new("a.rs", MACRO_GUARD)], &["a.rs:2"]),
    Case::new("macro_rules body", &[Source::new("a.rs", MACRO_RULES)], &["a.rs:3"]),
    Case::new("repeat macro", &[Source::new("a.rs", REPEAT_MACRO)], &["a.rs:2"]),
    Case::new("re-export through ports", &[Source::new("a.rs", THROUGH_PORTS)], &["a.rs:5"]),
    Case::new("import in a sibling module", &[Source::new("a.rs", SIBLING_IMPORT)], &[]),
    Case::new(
        "ports in a test target",
        &[Source::new("crates/x/tests/ports.rs", IMPORTED)],
        &["crates/x/tests/ports.rs:3"],
    ),
];

const PACKAGES: [(&str, usize); 9] = [
    ("meta-ports", 0),
    ("ports", 0),
    ("meta_ports", 0),
    ("meta-support", 1),
    ("meta-run", 1),
    ("meta-reports", 1),
    ("transports", 1),
    ("exports", 1),
    ("supports", 1),
];

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

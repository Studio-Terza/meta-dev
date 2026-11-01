use std::error::Error;
use std::fs;

use lint_core::Lint;
use lint_process_starters::{CommandPaths, ProcessStarters, Scope};

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

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

const CASES: [Case; 31] = [
    Case::new(
        Title::new("test module"),
        &[Source::new(Located::new("a.rs"), Text::new(TEST_MODULE))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("integration test"),
        &[Source::new(Located::new("tests/spawns.rs"), Text::new(IMPORTED))],
        Flagged::new(&["tests/spawns.rs:3"]),
    ),
    Case::new(
        Title::new("module alias"),
        &[Source::new(Located::new("a.rs"), Text::new(ALIASED))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("ports module file"),
        &[Source::new(Located::new("crates/x/src/ports/run.rs"), Text::new(IMPORTED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("ports module root"),
        &[Source::new(Located::new("crates/x/src/ports.rs"), Text::new(IMPORTED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inline ports module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE_PORTS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inline other module"),
        &[Source::new(Located::new("a.rs"), Text::new(INLINE_OTHER))],
        Flagged::new(&["a.rs:4"]),
    ),
    Case::new(
        Title::new("name only resembling ports"),
        &[Source::new(Located::new("crates/x/src/support/run.rs"), Text::new(IMPORTED))],
        Flagged::new(&["crates/x/src/support/run.rs:3"]),
    ),
    Case::new(
        Title::new("use after the code"),
        &[Source::new(Located::new("a.rs"), Text::new(USE_AFTER))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("self renamed"),
        &[Source::new(Located::new("a.rs"), Text::new(SELF_RENAMED))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("function value"),
        &[Source::new(Located::new("a.rs"), Text::new(FUNCTION_VALUE))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("type alias"),
        &[Source::new(Located::new("a.rs"), Text::new(TYPE_ALIAS))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("re-export in the crate root"),
        &[
            Source::new(Located::new("crates/x/src/lib.rs"), Text::new(ROOT_EXPORT)),
            Source::new(Located::new("crates/x/src/user.rs"), Text::new(RE_EXPORTED)),
        ],
        Flagged::new(&["crates/x/src/user.rs:2"]),
    ),
    Case::new(
        Title::new("qualified self"),
        &[Source::new(Located::new("a.rs"), Text::new(QUALIFIED_SELF))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("macro guard"),
        &[Source::new(Located::new("a.rs"), Text::new(MACRO_GUARD))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("macro_rules body"),
        &[Source::new(Located::new("a.rs"), Text::new(MACRO_RULES))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("repeat macro"),
        &[Source::new(Located::new("a.rs"), Text::new(REPEAT_MACRO))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("re-export through ports"),
        &[Source::new(Located::new("a.rs"), Text::new(THROUGH_PORTS))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("import in a sibling module"),
        &[Source::new(Located::new("a.rs"), Text::new(SIBLING_IMPORT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("ports in a test target"),
        &[Source::new(Located::new("crates/x/tests/ports.rs"), Text::new(IMPORTED))],
        Flagged::new(&["crates/x/tests/ports.rs:3"]),
    ),
    Case::new(
        Title::new("inside a macro argument"),
        &[Source::new(Located::new("a.rs"), Text::new(IN_MACRO))],
        Flagged::new(&["a.rs:2"]),
    ),
    Case::new(
        Title::new("a chain over lines"),
        &[Source::new(Located::new("a.rs"), Text::new(SPLIT_CHAIN))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("a chain with no start"),
        &[Source::new(Located::new("a.rs"), Text::new(CONFIGURED_ONLY))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("a foreign renamed Command"),
        &[Source::new(Located::new("a.rs"), Text::new(FOREIGN_RENAME))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("another renamed process type"),
        &[Source::new(Located::new("a.rs"), Text::new(OTHER_RENAME))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a process glob"),
        &[Source::new(Located::new("a.rs"), Text::new(GLOB))],
        Flagged::new(&["a.rs:3"]),
    ),
    Case::new(
        Title::new("a foreign glob"),
        &[Source::new(Located::new("a.rs"), Text::new(FOREIGN_GLOB))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("three segments"),
        &[Source::new(Located::new("a.rs"), Text::new(THREE_SEGMENTS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("another process type"),
        &[Source::new(Located::new("a.rs"), Text::new(OTHER_TYPE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("new of another path"),
        &[Source::new(Located::new("a.rs"), Text::new(NEW_OF_ANOTHER))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("another Command constructor"),
        &[Source::new(Located::new("a.rs"), Text::new(OTHER_CONSTRUCTOR))],
        Flagged::new(&[]),
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
    assert_eq!(Sources::mismatches::<ProcessStarters>(&CASES), Lines::default());
}

#[test]
fn a_crate_whose_name_ends_in_ports_may_start_processes() -> Result<(), Box<dyn Error>> {
    let mut wrong = Vec::new();
    for (name, flagged) in PACKAGES {
        let root = tempfile::tempdir()?;
        fs::create_dir(root.path().join("src"))?;
        fs::write(root.path().join("Cargo.toml"), format!("[package]\nname = \"{name}\"\n"))?;
        let file = Sources::file(root.path().join("src/lib.rs"), Text::new(IMPORTED));
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
    let ported = Sources::file("crates/x/src/ports/run.rs", Text::new(IMPORTED));
    let inline = Sources::file("a.rs", Text::new(INLINE_PORTS));
    let other = Sources::file("b.rs", Text::new(OTHER_COMMAND));
    let counts: Vec<usize> =
        [ported, inline, other].iter().map(|file| CommandPaths::found(file, Scope::Everywhere).len()).collect();
    assert_eq!(counts, vec![1, 1, 0]);
}

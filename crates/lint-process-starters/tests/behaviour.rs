use lint_process_starters::ProcessStarters;

use lint_fixtures::{Case, Source, Sources};

const IMPORTED: &str = "use std::process::Command;\nfn f() {\n    Command::new(\"ls\").status();\n}\n";
const QUALIFIED: &str = "fn f() {\n    std::process::Command::new(\"ls\").spawn();\n}\n";
const RENAMED: &str = "use std::process::Command as Run;\nfn f() {\n    Run::new(\"ls\").output();\n}\n";
const BUILT_ONLY: &str = "use std::process::Command;\nfn f() -> Command {\n    Command::new(\"ls\")\n}\n";
const OTHER_COMMAND: &str = "use clap::Command;\nfn f() {\n    Command::new(\"ls\").status();\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    use std::process::Command;\n    fn f() {\n        Command::new(\"ls\").status();\n    }\n}\n";

const ALIASED: &str = "use std::process as p;\nfn f() {\n    p::Command::new(\"ls\").status();\n}\n";

const CASES: [Case; 8] = [
    Case::new("imported", &[Source::new("a.rs", IMPORTED)], &["a.rs:3"]),
    Case::new("qualified", &[Source::new("a.rs", QUALIFIED)], &["a.rs:2"]),
    Case::new("renamed", &[Source::new("a.rs", RENAMED)], &["a.rs:3"]),
    Case::new("built only", &[Source::new("a.rs", BUILT_ONLY)], &["a.rs:3"]),
    Case::new("other command", &[Source::new("a.rs", OTHER_COMMAND)], &[]),
    Case::new("test module", &[Source::new("a.rs", TEST_MODULE)], &["a.rs:5"]),
    Case::new("integration test", &[Source::new("tests/spawns.rs", IMPORTED)], &["tests/spawns.rs:3"]),
    Case::new("module alias", &[Source::new("a.rs", ALIASED)], &["a.rs:3"]),
];

#[test]
fn starting_a_process_is_flagged() {
    assert_eq!(Sources::mismatches::<ProcessStarters>(&CASES), Vec::<String>::new());
}

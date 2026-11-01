use lints::ProcessStarters;

use crate::common::{Case, Source, Sources};

const IMPORTED: &str = "use std::process::Command;\nfn f() {\n    Command::new(\"ls\").status();\n}\n";
const QUALIFIED: &str = "fn f() {\n    std::process::Command::new(\"ls\").spawn();\n}\n";
const RENAMED: &str = "use std::process::Command as Run;\nfn f() {\n    Run::new(\"ls\").output();\n}\n";
const BUILT_ONLY: &str = "use std::process::Command;\nfn f() -> Command {\n    Command::new(\"ls\")\n}\n";
const OTHER_COMMAND: &str = "use clap::Command;\nfn f() {\n    Command::new(\"ls\").status();\n}\n";
const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {\n    use std::process::Command;\n    fn f() {\n        Command::new(\"ls\").status();\n    }\n}\n";

const ALIASED: &str = "use std::process as p;\nfn f() {\n    p::Command::new(\"ls\").status();\n}\n";

const CASES: [Case; 8] = [
    Case { name: "imported", sources: &[Source::new("a.rs", IMPORTED)], flagged: &["a.rs:3"] },
    Case { name: "qualified", sources: &[Source::new("a.rs", QUALIFIED)], flagged: &["a.rs:2"] },
    Case { name: "renamed", sources: &[Source::new("a.rs", RENAMED)], flagged: &["a.rs:3"] },
    Case { name: "built only", sources: &[Source::new("a.rs", BUILT_ONLY)], flagged: &["a.rs:3"] },
    Case { name: "other command", sources: &[Source::new("a.rs", OTHER_COMMAND)], flagged: &[] },
    Case { name: "test module", sources: &[Source::new("a.rs", TEST_MODULE)], flagged: &["a.rs:5"] },
    Case {
        name: "integration test",
        sources: &[Source::new("tests/spawns.rs", IMPORTED)],
        flagged: &["tests/spawns.rs:3"],
    },
    Case { name: "module alias", sources: &[Source::new("a.rs", ALIASED)], flagged: &["a.rs:3"] },
];

#[test]
fn starting_a_process_is_flagged() {
    assert_eq!(Sources::mismatches::<ProcessStarters>(&CASES), Vec::<String>::new());
}

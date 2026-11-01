use std::ffi::OsString;
use std::process::ExitCode;

use reach::Cli;

const WORKSPACE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/metadata/workspace.json");
const PATCHED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/metadata/patched.json");
const ROOTED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/metadata/rooted.json");
const READING: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/metadata/reading.json");
const BROKEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/metadata/broken.json");
const OUTSIDE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/metadata/outside.json");
const ALL: [&str; 5] = ["core", "leaf", "middle", "serde-user", "tested-through"];
const LEAF_REPORT: &str = "REACH=1 of 5 crates\nCARGO_SCOPE=-p leaf\nMUTANTS_SCOPE=--test-package=leaf\n";
const TWO_REPORT: &str = "REACH=2 of 5 crates\nCARGO_SCOPE=-p middle -p tested-through\n\
    MUTANTS_SCOPE=--test-package=middle --test-package=tested-through\n";
const EMPTY_REPORT: &str = "REACH=0 of 5 crates\nCARGO_SCOPE=\nMUTANTS_SCOPE=\n";
const HISTORY_LEAF: &str = "reach 1 of 5\tfan-in core=2 middle=1 leaf=0 serde-user=0 tested-through=0\n";
const HISTORY_CORE: &str = "reach 3 of 5\tfan-in core=2 middle=1 leaf=0 serde-user=0 tested-through=0\n";
const HISTORY_NONE: &str = "reach 0 of 5\tfan-in core=2 middle=1 leaf=0 serde-user=0 tested-through=0\n";
const FAN_IN_REPORT: &str = "fan-in, crates that depend on each crate, directly or not:\n\
    2 core\n1 middle\n0 leaf\n0 serde-user\n0 tested-through\n";

#[derive(Debug, PartialEq)]
struct Answer {
    exit: ExitCode,
    out: String,
    err: String,
}

impl Answer {
    fn to(arguments: &[&str]) -> Answer {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let exit = Cli::answer(arguments.iter().map(OsString::from), &mut out, &mut err);
        Answer {
            exit,
            out: String::from_utf8(out).unwrap_or_default(),
            err: String::from_utf8(err).unwrap_or_default(),
        }
    }

    fn reached_by(changed: &[&str]) -> String {
        let arguments = ["affected", WORKSPACE].iter().chain(changed).copied().collect::<Vec<&str>>();
        Answer::to(&arguments).out
    }

    fn sources_of(roots: &[&str]) -> String {
        let arguments = ["sources", WORKSPACE].iter().chain(roots).copied().collect::<Vec<&str>>();
        Answer::to(&arguments).out
    }

    fn history_by(changed: &[&str]) -> String {
        let arguments = ["history", WORKSPACE].iter().chain(changed).copied().collect::<Vec<&str>>();
        Answer::to(&arguments).out
    }

    fn reached_in(metadata: &str, changed: &[&str]) -> String {
        let arguments = ["affected", metadata].iter().chain(changed).copied().collect::<Vec<&str>>();
        Answer::to(&arguments).out
    }

    fn widened_by(changed: &[&str]) -> String {
        let arguments = ["widened", WORKSPACE].iter().chain(changed).copied().collect::<Vec<&str>>();
        Answer::to(&arguments).out
    }

    fn crates(names: &[&str]) -> String {
        let flags = |flag: &str| names.iter().map(|name| format!("{flag}{name}")).collect::<Vec<String>>().join(" ");
        format!(
            "REACH={} of 5 crates\nCARGO_SCOPE={}\nMUTANTS_SCOPE={}\n",
            names.len(),
            flags("-p "),
            flags("--test-package=")
        )
    }

    fn refused(arguments: &[&str]) -> String {
        let answer = Answer::to(arguments);
        if answer.exit == ExitCode::FAILURE && answer.out.is_empty() { answer.err } else { String::new() }
    }
}

laws::table!(
    a_widened_report_keeps_the_reach_for_mutants_but_runs_cargo_on_the_whole_workspace,
    Answer::widened_by,
    [
        ["crates/leaf/src/lib.rs"] => "REACH=1 of 5 crates\nCARGO_SCOPE=--workspace\nMUTANTS_SCOPE=--test-package=leaf\n",
        ["README.md"] => "REACH=0 of 5 crates\nCARGO_SCOPE=--workspace\nMUTANTS_SCOPE=\n",
    ]
);

laws::table!(
    a_report_gives_the_share_and_both_scopes,
    Answer::reached_by,
    [
        ["crates/leaf/src/lib.rs"] => LEAF_REPORT,
        ["crates/middle/src/lib.rs"] => TWO_REPORT,
        ["README.md"] => EMPTY_REPORT,
    ]
);

laws::table!(
    a_change_reaches_its_crate_and_every_crate_depending_on_it,
    Answer::reached_by,
    [
        ["crates/tested-through/tests/behaviour.rs"] => Answer::crates(&["tested-through"]),
        ["crates/middle/compile_fail.md"] => Answer::crates(&["middle", "tested-through"]),
        ["crates/leaf/a.rs", "crates/middle/b.rs"] => Answer::crates(&["leaf", "middle", "tested-through"]),
        ["crates/leaf/a.rs", "README.md"] => Answer::crates(&["leaf"]),
    ]
);

laws::table!(
    a_change_to_a_file_of_no_crate_reaches_everything,
    Answer::reached_by,
    [
        ["Cargo.toml"] => Answer::crates(&ALL),
        ["Cargo.lock"] => Answer::crates(&ALL),
        ["crates/core/Cargo.toml"] => Answer::crates(&ALL),
        ["crates/leaf/Cargo.toml"] => Answer::crates(&ALL),
        ["rust-toolchain.toml"] => Answer::crates(&ALL),
        ["clippy.toml"] => Answer::crates(&ALL),
        [".cargo/mutants.toml"] => Answer::crates(&ALL),
        [".github/workflows/ci.yml"] => Answer::crates(&ALL),
        ["ci/limits.toml"] => Answer::crates(&ALL),
        ["crates/deleted/src/lib.rs"] => Answer::crates(&ALL),
        ["README.md", "Cargo.lock"] => Answer::crates(&ALL),
    ]
);

#[test]
fn a_dependency_served_by_a_patch_is_still_an_edge() {
    let reached = Answer::reached_in(PATCHED, &["crates/a/src/lib.rs"]);
    assert_eq!(
        reached,
        "REACH=2 of 2 crates\nCARGO_SCOPE=-p a -p b\nMUTANTS_SCOPE=--test-package=a --test-package=b\n"
    );
}

#[test]
fn a_package_at_the_workspace_root_does_not_own_every_path() {
    let everything =
        "REACH=2 of 2 crates\nCARGO_SCOPE=-p leaf -p root\nMUTANTS_SCOPE=--test-package=leaf --test-package=root\n";
    for changed in ["Cargo.lock", "rust-toolchain.toml", "crates/deleted/src/lib.rs", "src/lib.rs"] {
        assert_eq!(Answer::reached_in(ROOTED, &[changed]), everything, "{changed}");
    }
}

#[test]
fn a_crate_that_declares_what_it_reads_is_reached_by_changes_there() {
    let runner_and_tool =
        "REACH=2 of 3 crates\nCARGO_SCOPE=-p runner -p tool\nMUTANTS_SCOPE=--test-package=runner --test-package=tool\n";
    assert_eq!(
        Answer::reached_in(READING, &["crates/leaf/src/lib.rs"]),
        "REACH=3 of 3 crates\nCARGO_SCOPE=-p leaf -p runner -p tool\nMUTANTS_SCOPE=--test-package=leaf --test-package=runner --test-package=tool\n"
    );
    assert_eq!(Answer::reached_in(READING, &["crates/runner/src/lib.rs"]), runner_and_tool);
    assert_eq!(
        Answer::reached_in(READING, &["tool/src/lib.rs"]),
        "REACH=1 of 3 crates\nCARGO_SCOPE=-p tool\nMUTANTS_SCOPE=--test-package=tool\n"
    );
}

laws::table!(
    documents_outside_the_crates_reach_nothing,
    Answer::reached_by,
    [
        ["README.md"] => EMPTY_REPORT,
        ["specs/segmentation.md"] => EMPTY_REPORT,
        [] => EMPTY_REPORT,
    ]
);

laws::table!(
    every_unusable_input_fails_closed_and_says_why,
    Answer::refused,
    [
        [] => "usage: reach affected <metadata> [<changed file>...] | reach widened <metadata> [<changed file>...] | reach history <metadata> [<changed file>...] | reach fan-in <metadata> | reach sources <metadata> <crate>...\n",
        ["affected"] => "usage: reach affected <metadata> [<changed file>...] | reach widened <metadata> [<changed file>...] | reach history <metadata> [<changed file>...] | reach fan-in <metadata> | reach sources <metadata> <crate>...\n",
        ["other", WORKSPACE] => "usage: reach affected <metadata> [<changed file>...] | reach widened <metadata> [<changed file>...] | reach history <metadata> [<changed file>...] | reach fan-in <metadata> | reach sources <metadata> <crate>...\n",
        ["fan-in", OUTSIDE] => "/x/Cargo.toml is not inside the workspace\n",
    ]
);

#[test]
fn fan_in_counts_every_crate_a_crate_reaches_most_first() {
    let answer = Answer::to(&["fan-in", WORKSPACE]);
    assert_eq!(answer, Answer { exit: ExitCode::SUCCESS, out: FAN_IN_REPORT.to_owned(), err: String::new() });
}

const SOURCES_OF_MIDDLE: &str =
    "crates/core/Cargo.toml\ncrates/core/src/**\ncrates/middle/Cargo.toml\ncrates/middle/src/**\n";
const SOURCES_OF_TESTED_THROUGH: &str = "crates/core/Cargo.toml\ncrates/core/src/**\ncrates/middle/Cargo.toml\n\
    crates/middle/src/**\ncrates/tested-through/Cargo.toml\ncrates/tested-through/src/**\n";

laws::table!(
    the_sources_of_roots_are_their_own_and_those_of_every_crate_they_depend_on,
    Answer::sources_of,
    [
        ["middle"] => SOURCES_OF_MIDDLE,
        ["tested-through"] => SOURCES_OF_TESTED_THROUGH,
        ["leaf"] => "crates/leaf/Cargo.toml\ncrates/leaf/src/**\n",
        ["core", "leaf"] => "crates/core/Cargo.toml\ncrates/core/src/**\ncrates/leaf/Cargo.toml\ncrates/leaf/src/**\n",
        ["absent"] => "",
        [] => "",
    ]
);

laws::table!(
    a_history_line_gives_the_reach_and_every_fan_in,
    Answer::history_by,
    [
        ["crates/leaf/src/lib.rs"] => HISTORY_LEAF,
        ["crates/core/src/lib.rs"] => HISTORY_CORE,
        ["README.md"] => HISTORY_NONE,
    ]
);

#[test]
fn metadata_that_cannot_be_read_or_parsed_is_refused() {
    assert!(Answer::refused(&["fan-in", "/no/such/metadata.json"]).contains("cannot read /no/such/metadata.json"));
    assert!(Answer::refused(&["affected", BROKEN]).contains("cannot parse cargo metadata"));
}

use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use file_ports::{Bytes, Files, Memory};
use reach::Cli;

const WORKSPACE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/metadata/workspace.json");
const PATCHED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/metadata/patched.json");
const ROOTED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/metadata/rooted.json");
const READING: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/metadata/reading.json");
const BROKEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/metadata/broken.json");
const HUBS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/metadata/hubs.json");
const HALF: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/metadata/half.json");
const OUTSIDE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/metadata/outside.json");
const ALL: [&str; 5] = ["core", "leaf", "middle", "serde-user", "tested-through"];
const LEAF_REPORT: [&str; 3] = ["REACH=1 of 5 crates", "BUILD_SCOPE=-p leaf", "MUTANTS_SCOPE=--test-package=leaf"];
const TWO_REPORT: [&str; 3] = [
    "REACH=2 of 5 crates",
    "BUILD_SCOPE=-p middle -p tested-through",
    "MUTANTS_SCOPE=--test-package=middle --test-package=tested-through",
];
const EMPTY_REPORT: [&str; 3] = ["REACH=0 of 5 crates", "BUILD_SCOPE=", "MUTANTS_SCOPE="];
const HISTORY_LEAF: [&str; 1] = ["reach 1 of 5\tfan-in core=2 middle=1 leaf=0 serde-user=0 tested-through=0"];
const HISTORY_CORE: [&str; 1] = ["reach 3 of 5\tfan-in core=2 middle=1 leaf=0 serde-user=0 tested-through=0"];
const HISTORY_NONE: [&str; 1] = ["reach 0 of 5\tfan-in core=2 middle=1 leaf=0 serde-user=0 tested-through=0"];
const FAN_IN_REPORT: [&str; 18] = [
    "fan-in, crates that depend on each crate, directly or not, of 5 workspace crates:",
    "2 of 5, 40%: core",
    "1 of 5, 20%: middle",
    "0 of 5, 0%: leaf",
    "0 of 5, 0%: serde-user",
    "0 of 5, 0%: tested-through",
    "crates per tenth of the workspace depending on them:",
    "0-9% 3",
    "10-19% 0",
    "20-29% 1",
    "30-39% 0",
    "40-49% 1",
    "50-59% 0",
    "60-69% 0",
    "70-79% 0",
    "80-89% 0",
    "90-99% 0",
    "median 0 of 5, 0%; largest 2 of 5, 40%: core",
];
const WORKSPACE_MANIFEST: &str = "[workspace]\nmembers = [\"crates/*\"]\n";
const BRANCHY: &str = "pub fn pick(x: u32) -> u32 {\n    if x > 1 { 2 } else if x > 3 { 4 } else { 5 }\n}\n";
const ONE_BRANCH: &str = "pub fn pick(x: u32) -> u32 {\n    if x > 1 { 2 } else { 5 }\n}\n";
const STRAIGHT: &str = "pub fn pick(x: u32) -> u32 {\n    x\n}\n";
const HUBS_HEADLINE: &str = "report only, hub thickness: the logic each crate holds, its shipped functions' cyclomatic complexity above one summed, weighted by the crates that depend on it; a crate many depend on should hold types, traits and interfaces, with the logic in the leaves:\n";
const HUB_CORE: &str = "crate core: logic 2, depended on by 2 of 3, 66%; weighted 4; move the logic into leaf crates and keep types, traits and interfaces here\n";
const HUB_LEAF: &str = "crate leaf: logic 1, depended on by 0 of 3, 0%; weighted 0; move the logic into leaf crates and keep types, traits and interfaces here\n";
const HUB_OTHER: &str = "crate other-leaf: logic 0, depended on by 0 of 3, 0%; weighted 0; move the logic into leaf crates and keep types, traits and interfaces here\n";
const HUB_HALF: &str = "crate core: logic 2, depended on by 3 of 6, 50%; weighted 6; move the logic into leaf crates and keep types, traits and interfaces here\n";
const HUB_NO_MAJORITY: &str = "logic in crates more than half the workspace depends on: 0 of 2, 0%\n";
const HUB_SHARE: &str = "logic in crates more than half the workspace depends on: 2 of 3, 66%\n";

#[derive(Debug, PartialEq)]
struct Answer {
    exit: ExitCode,
    out: String,
    err: String,
}

impl Answer {
    fn to(arguments: &[&str]) -> Answer {
        Answer::within(&Repository::metadata(), arguments)
    }

    fn said(arguments: &[&str]) -> Vec<String> {
        Text::owned(&Answer::to(arguments).out)
    }

    fn within(files: &impl Files, arguments: &[&str]) -> Answer {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let exit = Cli::answer(arguments.iter().map(OsString::from), files, &mut out, &mut err);
        Answer {
            exit,
            out: String::from_utf8(out).unwrap_or_default(),
            err: String::from_utf8(err).unwrap_or_default(),
        }
    }

    fn crates(names: &[&str]) -> Vec<String> {
        let flags = |flag: &str| names.iter().map(|name| format!("{flag}{name}")).collect::<Vec<String>>().join(" ");
        vec![
            format!("REACH={} of 5 crates", names.len()),
            format!("BUILD_SCOPE={}", flags("-p ")),
            format!("MUTANTS_SCOPE={}", flags("--test-package=")),
        ]
    }

    fn refusal(arguments: &[&str]) -> Vec<String> {
        Text::owned(&Answer::refused(arguments))
    }

    fn refused(arguments: &[&str]) -> String {
        let answer = Answer::to(arguments);
        if answer.exit == ExitCode::FAILURE && answer.out.is_empty() { answer.err } else { String::new() }
    }
}

laws::table!(
    a_widened_report_keeps_the_reach_for_mutants_but_runs_cargo_on_the_whole_workspace,
    Answer::said,
    [
        ["widened", WORKSPACE, "crates/leaf/src/lib.rs"] => vec!["REACH=1 of 5 crates", "BUILD_SCOPE=--workspace", "MUTANTS_SCOPE=--test-package=leaf"],
        ["widened", WORKSPACE, "README.md"] => vec!["REACH=0 of 5 crates", "BUILD_SCOPE=--workspace", "MUTANTS_SCOPE="],
    ]
);

laws::table!(
    a_report_gives_the_share_and_both_scopes,
    Answer::said,
    [
        ["affected", WORKSPACE, "crates/leaf/src/lib.rs"] => LEAF_REPORT.to_vec(),
        ["affected", WORKSPACE, "crates/middle/src/lib.rs"] => TWO_REPORT.to_vec(),
        ["affected", WORKSPACE, "README.md"] => EMPTY_REPORT.to_vec(),
    ]
);

laws::table!(
    a_change_reaches_its_crate_and_every_crate_depending_on_it,
    Answer::said,
    [
        ["affected", WORKSPACE, "crates/tested-through/tests/behaviour.rs"] => Answer::crates(&["tested-through"]),
        ["affected", WORKSPACE, "crates/middle/compile_fail.md"] => Answer::crates(&["middle", "tested-through"]),
        ["affected", WORKSPACE, "crates/leaf/a.rs", "crates/middle/b.rs"] => Answer::crates(&["leaf", "middle", "tested-through"]),
        ["affected", WORKSPACE, "crates/leaf/a.rs", "README.md"] => Answer::crates(&["leaf"]),
    ]
);

laws::table!(
    a_change_to_a_file_of_no_crate_reaches_everything,
    Answer::said,
    [
        ["affected", WORKSPACE, "Cargo.toml"] => Answer::crates(&ALL),
        ["affected", WORKSPACE, "Cargo.lock"] => Answer::crates(&ALL),
        ["affected", WORKSPACE, "crates/core/Cargo.toml"] => Answer::crates(&ALL),
        ["affected", WORKSPACE, "crates/leaf/Cargo.toml"] => Answer::crates(&ALL),
        ["affected", WORKSPACE, "rust-toolchain.toml"] => Answer::crates(&ALL),
        ["affected", WORKSPACE, "clippy.toml"] => Answer::crates(&ALL),
        ["affected", WORKSPACE, ".cargo/mutants.toml"] => Answer::crates(&ALL),
        ["affected", WORKSPACE, ".github/workflows/ci.yml"] => Answer::crates(&ALL),
        ["affected", WORKSPACE, "ci/limits.toml"] => Answer::crates(&ALL),
        ["affected", WORKSPACE, "crates/deleted/src/lib.rs"] => Answer::crates(&ALL),
        ["affected", WORKSPACE, "README.md", "Cargo.lock"] => Answer::crates(&ALL),
    ]
);

#[test]
fn a_dependency_served_by_a_patch_is_still_an_edge() {
    let reached = Answer::said(&["affected", PATCHED, "crates/a/src/lib.rs"]);
    assert_eq!(
        reached,
        ["REACH=2 of 2 crates", "BUILD_SCOPE=-p a -p b", "MUTANTS_SCOPE=--test-package=a --test-package=b"]
    );
}

#[test]
fn a_package_at_the_workspace_root_does_not_own_every_path() {
    let everything =
        ["REACH=2 of 2 crates", "BUILD_SCOPE=-p leaf -p root", "MUTANTS_SCOPE=--test-package=leaf --test-package=root"];
    for changed in ["Cargo.lock", "rust-toolchain.toml", "crates/deleted/src/lib.rs", "src/lib.rs"] {
        assert_eq!(Answer::said(&["affected", ROOTED, changed]), everything, "{changed}");
    }
}

#[test]
fn a_crate_that_declares_what_it_reads_is_reached_by_changes_there() {
    let runner_and_tool = [
        "REACH=2 of 3 crates",
        "BUILD_SCOPE=-p runner -p tool",
        "MUTANTS_SCOPE=--test-package=runner --test-package=tool",
    ];
    assert_eq!(
        Answer::said(&["affected", READING, "crates/leaf/src/lib.rs"]),
        [
            "REACH=3 of 3 crates",
            "BUILD_SCOPE=-p leaf -p runner -p tool",
            "MUTANTS_SCOPE=--test-package=leaf --test-package=runner --test-package=tool"
        ]
    );
    assert_eq!(Answer::said(&["affected", READING, "crates/runner/src/lib.rs"]), runner_and_tool);
    assert_eq!(
        Answer::said(&["affected", READING, "tool/src/lib.rs"]),
        ["REACH=1 of 3 crates", "BUILD_SCOPE=-p tool", "MUTANTS_SCOPE=--test-package=tool"]
    );
}

laws::table!(
    documents_outside_the_crates_reach_nothing,
    Answer::said,
    [
        ["affected", WORKSPACE, "README.md"] => EMPTY_REPORT.to_vec(),
        ["affected", WORKSPACE, "specs/segmentation.md"] => EMPTY_REPORT.to_vec(),
        ["affected", WORKSPACE] => EMPTY_REPORT.to_vec(),
    ]
);

laws::table!(
    every_unusable_input_fails_closed_and_says_why,
    Answer::refusal,
    [
        [] => vec!["usage: reach affected <metadata> [<changed file>...] | reach widened <metadata> [<changed file>...] | reach history <metadata> [<changed file>...] | reach fan-in <metadata> | reach hubs <metadata> <repository> | reach sources <metadata> <crate>... | reach beyond <metadata> <produced file> [<changed file>...]"],
        ["affected"] => vec!["usage: reach affected <metadata> [<changed file>...] | reach widened <metadata> [<changed file>...] | reach history <metadata> [<changed file>...] | reach fan-in <metadata> | reach hubs <metadata> <repository> | reach sources <metadata> <crate>... | reach beyond <metadata> <produced file> [<changed file>...]"],
        ["other", WORKSPACE] => vec!["usage: reach affected <metadata> [<changed file>...] | reach widened <metadata> [<changed file>...] | reach history <metadata> [<changed file>...] | reach fan-in <metadata> | reach hubs <metadata> <repository> | reach sources <metadata> <crate>... | reach beyond <metadata> <produced file> [<changed file>...]"],
        ["beyond", WORKSPACE] => vec!["usage: reach affected <metadata> [<changed file>...] | reach widened <metadata> [<changed file>...] | reach history <metadata> [<changed file>...] | reach fan-in <metadata> | reach hubs <metadata> <repository> | reach sources <metadata> <crate>... | reach beyond <metadata> <produced file> [<changed file>...]"],
        ["fan-in", OUTSIDE] => vec!["/x/Cargo.toml is not inside the workspace"],
    ]
);

#[test]
fn fan_in_counts_every_crate_a_crate_reaches_most_first() {
    let answer = Answer::to(&["fan-in", WORKSPACE]);
    assert_eq!(
        (answer.exit, Text::lines(&answer.out), answer.err),
        (ExitCode::SUCCESS, FAN_IN_REPORT.to_vec(), String::new())
    );
}

const SOURCES_OF_MIDDLE: [&str; 4] =
    ["crates/core/Cargo.toml", "crates/core/src/**", "crates/middle/Cargo.toml", "crates/middle/src/**"];
const SOURCES_OF_TESTED_THROUGH: [&str; 6] = [
    "crates/core/Cargo.toml",
    "crates/core/src/**",
    "crates/middle/Cargo.toml",
    "crates/middle/src/**",
    "crates/tested-through/Cargo.toml",
    "crates/tested-through/src/**",
];

laws::table!(
    the_sources_of_roots_are_their_own_and_those_of_every_crate_they_depend_on,
    Answer::said,
    [
        ["sources", WORKSPACE, "middle"] => SOURCES_OF_MIDDLE.to_vec(),
        ["sources", WORKSPACE, "tested-through"] => SOURCES_OF_TESTED_THROUGH.to_vec(),
        ["sources", WORKSPACE, "leaf"] => vec!["crates/leaf/Cargo.toml", "crates/leaf/src/**"],
        ["sources", WORKSPACE, "core", "leaf"] => vec!["crates/core/Cargo.toml", "crates/core/src/**", "crates/leaf/Cargo.toml", "crates/leaf/src/**"],
        ["sources", WORKSPACE, "absent"] => Vec::<&str>::new(),
        ["sources", WORKSPACE] => Vec::<&str>::new(),
    ]
);

laws::table!(
    a_produced_file_is_beyond_the_changes_unless_they_reach_every_crate_owning_it,
    Answer::said,
    [
        ["beyond", WORKSPACE, "crates/leaf/tests/goldens/x.txt", "crates/leaf/src/lib.rs"] => Vec::<&str>::new(),
        ["beyond", WORKSPACE, "crates/tested-through/tests/goldens/x.txt", "crates/core/src/lib.rs"] => Vec::<&str>::new(),
        ["beyond", WORKSPACE, "crates/core/tests/goldens/x.txt", "crates/middle/src/lib.rs"] => vec!["crates/core/tests/goldens/x.txt"],
        ["beyond", WORKSPACE, "crates/leaf/tests/goldens/x.txt"] => vec!["crates/leaf/tests/goldens/x.txt"],
        ["beyond", WORKSPACE, "crates/leaf/tests/goldens/x.txt", "README.md"] => vec!["crates/leaf/tests/goldens/x.txt"],
        ["beyond", WORKSPACE, "tests/goldens/x.txt", "crates/leaf/src/lib.rs"] => vec!["tests/goldens/x.txt"],
        ["beyond", WORKSPACE, "tests/goldens/x.txt", "Cargo.lock"] => Vec::<&str>::new(),
        ["beyond", WORKSPACE, "specs/tests/goldens/x.txt", "Cargo.lock"] => vec!["specs/tests/goldens/x.txt"],
    ]
);

laws::table!(
    a_history_line_gives_the_reach_and_every_fan_in,
    Answer::said,
    [
        ["history", WORKSPACE, "crates/leaf/src/lib.rs"] => HISTORY_LEAF.to_vec(),
        ["history", WORKSPACE, "crates/core/src/lib.rs"] => HISTORY_CORE.to_vec(),
        ["history", WORKSPACE, "README.md"] => HISTORY_NONE.to_vec(),
    ]
);

#[test]
fn metadata_that_cannot_be_read_or_parsed_is_refused() {
    assert!(Answer::refused(&["fan-in", "/no/such/metadata.json"]).contains("cannot read /no/such/metadata.json"));
    assert!(Answer::refused(&["affected", BROKEN]).contains("cannot parse cargo metadata"));
}

struct Text;

impl Text {
    fn owned(text: &str) -> Vec<String> {
        text_lines::Lines::from(text).map(str::to_owned).collect()
    }

    fn lines(text: &impl AsRef<str>) -> Vec<&str> {
        text_lines::Lines::from(text.as_ref()).collect()
    }
}

#[test]
fn hubs_weigh_each_crates_logic_by_the_crates_depending_on_it_heaviest_first() -> Result<(), Box<dyn Error>> {
    let repository = Repository::new()?
        .with_crate("core", BRANCHY)?
        .with_crate("leaf", ONE_BRANCH)?
        .with_crate("other-leaf", STRAIGHT)?;
    let expected = format!(
        "{HUBS_HEADLINE}{}{HUB_CORE}{}{HUB_LEAF}{}{HUB_OTHER}{HUB_SHARE}",
        repository.site("core"),
        repository.site("leaf"),
        repository.site("other-leaf")
    );
    assert_eq!(repository.hubs(HUBS), Answer { exit: ExitCode::SUCCESS, out: expected, err: String::new() });
    Ok(())
}

#[test]
fn a_crate_exactly_half_the_workspace_depends_on_is_not_a_majority_hub() -> Result<(), Box<dyn Error>> {
    let repository = Repository::new()?.with_crate("core", BRANCHY)?;
    let expected = format!("{HUBS_HEADLINE}{}{HUB_HALF}{HUB_NO_MAJORITY}", repository.site("core"));
    assert_eq!(repository.hubs(HALF), Answer { exit: ExitCode::SUCCESS, out: expected, err: String::new() });
    Ok(())
}

#[test]
fn hubs_without_a_repository_is_a_usage_error() {
    assert!(Answer::refused(&["hubs", HUBS]).contains("reach hubs <metadata> <repository>"));
}

struct Repository {
    files: Memory,
    root: PathBuf,
}

impl Repository {
    fn metadata() -> Memory {
        golden::fixtures!("tests/metadata").unwrap_or_default()
    }

    fn new() -> Result<Repository, Box<dyn Error>> {
        let repository = Repository { files: Repository::metadata(), root: PathBuf::from("/repository") };
        repository.files.create_directories(&repository.root)?;
        repository.files.write(&repository.root.join("Cargo.toml"), Bytes::from(WORKSPACE_MANIFEST))?;
        Ok(repository)
    }

    fn with_crate(self, name: &str, source: &str) -> Result<Repository, Box<dyn Error>> {
        let directory = self.root.join("crates").join(name);
        self.files.create_directories(&directory.join("src"))?;
        self.files.write(&directory.join("Cargo.toml"), Bytes::from(&format!("[package]\nname = \"{name}\"\n")))?;
        self.files.write(&directory.join("src/lib.rs"), Bytes::from(source))?;
        Ok(self)
    }

    fn hubs(&self, metadata: &str) -> Answer {
        Answer::within(&self.files, &["hubs", metadata, &self.root.display().to_string()])
    }

    fn site(&self, name: &str) -> String {
        format!("{}: ", self.root.join("crates").join(name).join("src/lib.rs:1").display())
    }
}

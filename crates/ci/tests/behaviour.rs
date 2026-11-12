use std::cell::RefCell;
use std::error::Error;
use std::ffi::OsString;
use std::io::{self, Write};
use std::iter;
use std::num::NonZeroU8;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use ci::Cli;
use ci::ports::{Exit, Invocation, Ports, Run, Tee};
use file_ports::{Bytes, FileFailure, Files, Kind, Memory};

const METADATA: &str = include_str!("../../reach/tests/metadata/workspace.json");
const CLIPPY_FIXTURE: &str = include_str!("metadata/clippy_targets.json");
const TIMINGS_UNITS: &str = include_str!("timings/units.json");
const TIMINGS_METADATA: &str = include_str!("timings/metadata.json");
const EVERY_PLATFORM: &str = include_str!("timings/every-platform.json");
const HOST_PLATFORM: &str = include_str!("timings/host-platform.json");
const HOST: &str = "x86_64-unknown-linux-gnu";
const GRAPH_PATH: [&str; 6] = [
    "graph-path 2.50 seconds, the longest chain of clippy units, counting only units sccache never caches:",
    "0.40 ext-macro (build)",
    "0.70 core (check)",
    "0.60 app (check)",
    "0.80 app behaviour \"test\" (check-test)",
    "graph-work 3.95 seconds, the sum of the same units",
];
const CRATES_GENERATED: usize = 31;
const LINES_PRINTED: usize = 31;
const LEAF_REPORT: [&str; 3] = ["REACH=1 of 5 crates", "BUILD_SCOPE=-p leaf", "MUTANTS_SCOPE=--test-package=leaf"];
const TWO_REPORT: [&str; 3] = [
    "REACH=2 of 5 crates",
    "BUILD_SCOPE=-p middle -p tested-through",
    "MUTANTS_SCOPE=--test-package=middle --test-package=tested-through",
];
const EMPTY_REPORT: [&str; 3] = ["REACH=0 of 5 crates", "BUILD_SCOPE=", "MUTANTS_SCOPE="];
const EMPTY_NOTES: [&str; 2] = [
    "skipped: BUILD_SCOPE is empty, so the steps that use it do not run",
    "skipped: MUTANTS_SCOPE is empty, so the steps that use it do not run",
];
const EMPTY_REPORT_AND_NOTES: [&str; 5] = [
    "REACH=0 of 5 crates",
    "BUILD_SCOPE=",
    "MUTANTS_SCOPE=",
    "skipped: BUILD_SCOPE is empty, so the steps that use it do not run",
    "skipped: MUTANTS_SCOPE is empty, so the steps that use it do not run",
];
const HISTORY_LINE: &str = "abc123\treach 1 of 5";
const BUILD: [&str; 7] = ["build", "--locked", "-q", "--bin", "reach", "--bin", "lint"];
const ROUNDED_UP: Duration = Duration::from_millis(1_996);
const FAILED_WITH: NonZeroU8 = NonZeroU8::new(3).unwrap();

struct Reply {
    key: &'static str,
    text: String,
}

impl Reply {
    fn to(key: &'static str, text: &str) -> Reply {
        Reply { key, text: text.to_owned() }
    }
}

struct ExactReply {
    asked: Invocation,
    text: String,
}

struct Script {
    exit: Exit,
    elapsed: Duration,
    changed: String,
    metadata: String,
    answers: Vec<Reply>,
    exact: Vec<ExactReply>,
    failing: Option<&'static str>,
    logged_text: String,
    calls: RefCell<Vec<Invocation>>,
    logs: RefCell<Vec<PathBuf>>,
}

impl Script {
    fn new() -> Script {
        Script {
            exit: Exit::Success,
            elapsed: Duration::from_millis(1_500),
            changed: String::new(),
            metadata: METADATA.to_owned(),
            answers: Vec::new(),
            exact: Vec::new(),
            failing: None,
            logged_text: String::new(),
            calls: RefCell::new(Vec::new()),
            logs: RefCell::new(Vec::new()),
        }
    }

    fn refusing(program: &'static str) -> Script {
        Script { failing: Some(program), ..Script::new() }
    }

    fn answering(answers: Vec<Reply>) -> Script {
        Script { answers, ..Script::new() }
    }

    fn exiting(code: u8) -> Script {
        Script { exit: NonZeroU8::new(code).map_or(Exit::Success, Exit::Failure), ..Script::new() }
    }

    fn recorded(&self, invocation: &Invocation) -> io::Result<()> {
        self.calls.borrow_mut().push(invocation.clone());
        self.failing
            .filter(|program| invocation.program() == *program)
            .map_or(Ok(()), |_| Err(io::Error::other("boom")))
    }

    fn answer_to(&self, invocation: &Invocation) -> String {
        self.exact
            .iter()
            .find(|exact| exact.asked == *invocation)
            .map_or_else(|| self.keyed_answer(invocation), |exact| exact.text.clone())
    }

    fn keyed_answer(&self, invocation: &Invocation) -> String {
        let key = [
            invocation.program().to_string_lossy().as_ref(),
            " ",
            invocation.arguments().first().map_or_default(|first| first.to_string_lossy()).as_ref(),
        ]
        .concat();
        self.answers.iter().find(|reply| reply.key == key).map_or_else(
            || {
                Some(invocation.program())
                    .filter(|program| *program == "git")
                    .map_or_else(|| self.metadata.clone(), |_| self.changed.clone())
            },
            |reply| reply.text.clone(),
        )
    }
}

struct Fake<'world> {
    script: &'world Script,
    scratch: &'world Scratch,
}

impl Ports for Fake<'_> {
    type Files = Memory;

    fn files(&self) -> &Memory {
        &self.scratch.files
    }

    fn run(&self, invocation: &Invocation) -> io::Result<Run> {
        self.script.recorded(invocation)?;
        Ok(Run::new(self.script.exit, self.script.elapsed))
    }

    fn output(&self, invocation: &Invocation) -> io::Result<ci::ports::Text> {
        self.script.recorded(invocation)?;
        Ok(ci::ports::Text::new(self.script.answer_to(invocation)))
    }

    fn logged(&self, invocation: &Invocation, log: &Path) -> io::Result<Run> {
        self.script.recorded(invocation)?;
        self.script.logs.borrow_mut().push(log.to_path_buf());
        self.scratch.placed(log, &self.script.logged_text)?;
        Ok(Run::new(self.script.exit, self.script.elapsed))
    }

    fn runs(&self, invocations: &[Invocation]) -> Vec<io::Result<Run>> {
        invocations.iter().map(|invocation| self.run(invocation)).collect()
    }

    fn outputs(&self, invocations: &[Invocation]) -> Vec<io::Result<ci::ports::Text>> {
        invocations.iter().map(|invocation| self.output(invocation)).collect()
    }
}

#[derive(Debug, PartialEq)]
struct Answer {
    exit: ExitCode,
    out: String,
    err: String,
}

impl Answer {
    fn lines(&self) -> Printed<String> {
        Printed { exit: self.exit, out: Text::lines(&self.out), err: Text::lines(&self.err) }
    }

    fn to<Argument: AsRef<str>>(fake: &Fake<'_>, arguments: &[Argument]) -> Answer {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let exit =
            Cli::answer(arguments.iter().map(|argument| OsString::from(argument.as_ref())), fake, &mut out, &mut err);
        Answer {
            exit,
            out: String::from_utf8(out).unwrap_or_default(),
            err: String::from_utf8(err).unwrap_or_default(),
        }
    }

    fn refused(arguments: &[&str]) -> bool {
        let answer = Scratch::fresh().answer(&Script::new(), arguments);
        (answer.exit, answer.out.as_str(), answer.err.contains("usage: ci ")) == (ExitCode::FAILURE, "", true)
    }
}

struct Reach;

impl Reach {
    fn on(scratch: &Scratch, event: &str) -> Vec<String> {
        Reach::into(scratch, event, "")
    }

    fn into(scratch: &Scratch, event: &str, base: &str) -> Vec<String> {
        ["reach", &scratch.file("metadata.json"), event, base, &scratch.file("env"), &scratch.file("sum")]
            .map(str::to_owned)
            .to_vec()
    }
}

#[derive(Debug, PartialEq)]
struct Recorded {
    exit: ExitCode,
    lines: Vec<String>,
}

struct Sample;

impl Sample {
    fn timed_line(elapsed: &Duration) -> Vec<String> {
        let scratch = Scratch::fresh();
        let fake = Script { elapsed: *elapsed, ..Script::new() };
        scratch.answer(&fake, &["timed", &scratch.file("m.toml"), "clippy", "cargo"]);
        Text::owned(&scratch.read("m.toml"))
    }

    fn measured(exit: ExitCode, lines: &[&str]) -> Recorded {
        Recorded { exit, lines: lines.iter().map(|line| (*line).to_owned()).collect() }
    }

    fn libtest_line(output: &&str) -> Recorded {
        let scratch = Scratch::fresh();
        scratch.write("out.txt", output).ok();
        let answer = scratch.answer(
            &Script::new(),
            &["libtest-seconds", &scratch.file("m.toml"), "test-run", &scratch.file("out.txt")],
        );
        Recorded { exit: answer.exit, lines: Text::owned(&scratch.read("m.toml")) }
    }

    fn reach_report(&[base, changed]: &[&str; 2]) -> Vec<String> {
        let scratch = Scratch::fresh();
        let fake = Script { changed: changed.to_owned(), ..Script::new() };
        Text::owned(&scratch.answer(&fake, &Reach::into(&scratch, "pull_request", base)).out)
    }
}

laws::table!(
    a_stage_is_recorded_in_seconds_to_two_places,
    Sample::timed_line,
    [
        Duration::from_millis(1_500) => vec!["clippy = 1.50"],
        ROUNDED_UP => vec!["clippy = 2.00"],
        Duration::from_millis(4) => vec!["clippy = 0.00"],
        Duration::from_millis(1_004) => vec!["clippy = 1.00"],
        Duration::from_millis(1_006) => vec!["clippy = 1.01"],
        Duration::ZERO => vec!["clippy = 0.00"],
        Duration::from_secs(125) => vec!["clippy = 125.00"],
    ]
);

laws::table!(
    the_libtest_seconds_of_every_result_line_are_added,
    Sample::libtest_line,
    [
        "test result: ok. 3 passed; 0 failed; finished in 0.50s\n" => Sample::measured(ExitCode::SUCCESS, &["test-run = 0.50"]),
        "test result: ok. 1 passed; finished in 0.50s\ntest result: ok. 2 passed; finished in 1.25s\n" => Sample::measured(ExitCode::SUCCESS, &["test-run = 1.75"]),
        "running 3 tests\n...\ntest result: ok. finished in 2.00s\nfinished in 9.00s\n" => Sample::measured(ExitCode::SUCCESS, &["test-run = 2.00"]),
        "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s\n" => Sample::measured(ExitCode::SUCCESS, &["test-run = 0.50"]),
        "test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.25s\n" => Sample::measured(ExitCode::SUCCESS, &["test-run = 1.25"]),
        "test result: ok. 1 passed; 0 failed\n" => Sample::measured(ExitCode::FAILURE, &[]),
        "" => Sample::measured(ExitCode::FAILURE, &[]),
        "no result lines here\n" => Sample::measured(ExitCode::FAILURE, &[]),
        "test result: ok. finished in abc\n" => Sample::measured(ExitCode::FAILURE, &[]),
        "test result: ok. finished in 3s\n" => Sample::measured(ExitCode::SUCCESS, &["test-run = 3.00"]),
        "test result: ok. finished in 0.5\n" => Sample::measured(ExitCode::FAILURE, &[]),
        "test result: ok. finished in 1s in 2s\n" => Sample::measured(ExitCode::FAILURE, &[]),
        "  test result: ok. finished in 9s\n" => Sample::measured(ExitCode::FAILURE, &[]),
        "test results: ok. finished in 4s\n" => Sample::measured(ExitCode::FAILURE, &[]),
        "test result: ok. finished in\n" => Sample::measured(ExitCode::FAILURE, &[]),
        "test result: ok. finishedin 5s\n" => Sample::measured(ExitCode::FAILURE, &[]),
        "test result: ok. in 1.5.5s\n" => Sample::measured(ExitCode::FAILURE, &[]),
        "test resul: ok. finished in 5s\n" => Sample::measured(ExitCode::FAILURE, &[]),
        "test result: ok. finished  in\t7s\n" => Sample::measured(ExitCode::SUCCESS, &["test-run = 7.00"]),
        "test result: ok. finished in 0.004s\n" => Sample::measured(ExitCode::SUCCESS, &["test-run = 0.00"]),
        "test result: ok. finished in 2.00s\ntest result: ok. finished in abc\n" => Sample::measured(ExitCode::FAILURE, &[]),
    ]
);

laws::table!(
    the_scope_is_what_the_changed_files_reach,
    Sample::reach_report,
    [
        ["feature", "crates/leaf/src/lib.rs\n"] => LEAF_REPORT.to_vec(),
        ["feature", "crates/middle/src/lib.rs\n"] => TWO_REPORT.to_vec(),
        ["feature", "crates/middle/src/lib.rs\ncrates/middle/src/other.rs\n"] => TWO_REPORT.to_vec(),
        ["feature", "README.md\n"] => EMPTY_REPORT_AND_NOTES.to_vec(),
        ["feature", ""] => EMPTY_REPORT_AND_NOTES.to_vec(),
        ["project", "crates/leaf/src/lib.rs\n"] => LEAF_REPORT.to_vec(),
        ["projects/x", "crates/leaf/src/lib.rs\n"] => LEAF_REPORT.to_vec(),
        ["feature/project/x", "crates/leaf/src/lib.rs\n"] => LEAF_REPORT.to_vec(),
    ]
);

laws::table!(
    a_pull_request_into_main_runs_the_whole_workspace_but_reports_the_reach_for_mutants,
    Sample::reach_report,
    [
        ["main", "crates/leaf/src/lib.rs\n"] => vec!["REACH=1 of 5 crates", "BUILD_SCOPE=--workspace", "MUTANTS_SCOPE=--test-package=leaf"],
        ["main", "README.md\n"] => vec!["REACH=0 of 5 crates", "BUILD_SCOPE=--workspace", "MUTANTS_SCOPE=", "skipped: MUTANTS_SCOPE is empty, so the steps that use it do not run"],
    ]
);

laws::table!(
    a_pull_request_into_a_project_branch_runs_the_whole_workspace_like_one_into_main,
    Sample::reach_report,
    [
        ["project/transcript-design", "crates/leaf/src/lib.rs\n"] => vec!["REACH=1 of 5 crates", "BUILD_SCOPE=--workspace", "MUTANTS_SCOPE=--test-package=leaf"],
        ["project/x", "crates/middle/src/lib.rs\n"] => vec!["REACH=2 of 5 crates", "BUILD_SCOPE=--workspace", "MUTANTS_SCOPE=--test-package=middle --test-package=tested-through"],
        ["project/x", "README.md\n"] => vec!["REACH=0 of 5 crates", "BUILD_SCOPE=--workspace", "MUTANTS_SCOPE=", "skipped: MUTANTS_SCOPE is empty, so the steps that use it do not run"],
    ]
);

struct Clippy;

impl Clippy {
    fn invoked(scope: &[&str]) -> Vec<String> {
        let scratch = Scratch::fresh();
        let fake = Script { metadata: CLIPPY_FIXTURE.to_owned(), ..Script::new() };
        let measured = scratch.file("m.toml");
        scratch.answer(&fake, &[&["clippy-targets", measured.as_str(), "clippy"], scope].concat());
        let calls = fake.calls.borrow();
        calls
            .iter()
            .filter(|call| call.arguments().first().is_some_and(|first| first == "clippy"))
            .flat_map(|call| call.arguments().iter().skip(2))
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect()
    }

    fn refused(scope: &[&str]) -> Vec<String> {
        Clippy::refused_on(CLIPPY_FIXTURE, scope)
    }

    fn refused_on(metadata: &str, scope: &[&str]) -> Vec<String> {
        let scratch = Scratch::fresh();
        let fake = Script { metadata: metadata.to_owned(), ..Script::new() };
        let measured = scratch.file("m.toml");
        let answer = scratch.answer(&fake, &[&["clippy-targets", measured.as_str(), "clippy"], scope].concat());
        assert_eq!((answer.exit, answer.out.as_str(), scratch.read("m.toml").as_str()), (ExitCode::FAILURE, "", ""));
        assert!(fake.calls.borrow().iter().all(|call| call.arguments().first().is_none_or(|first| first != "clippy")));
        Text::lines(&answer.err)
    }
}

laws::table!(
    clippy_names_the_kinds_of_target_the_scope_has_and_always_the_tests,
    Clippy::invoked,
    [
        &["--workspace"][..] => vec!["--lib", "--bins", "--example", "showcase", "--bench", "speed", "--tests", "--workspace"],
        &["--all"][..] => vec!["--lib", "--bins", "--example", "showcase", "--bench", "speed", "--tests", "--all"],
        &["--workspace", "--timings", "--", "-D", "warnings"][..] => vec!["--lib", "--bins", "--example", "showcase", "--bench", "speed", "--tests", "--workspace", "--timings", "--", "-D", "warnings"],
        &["-p", "libonly"][..] => vec!["--lib", "--tests", "-p", "libonly"],
        &["-p", "binonly"][..] => vec!["--bins", "--tests", "-p", "binonly"],
        &["-p", "demo"][..] => vec!["--lib", "--example", "showcase", "--tests", "-p", "demo"],
        &["-p", "benched"][..] => vec!["--lib", "--bench", "speed", "--tests", "-p", "benched"],
        &["-p", "libonly", "-p", "benched"][..] => vec!["--lib", "--bench", "speed", "--tests", "-p", "libonly", "-p", "benched"],
        &["-p", "macros"][..] => vec!["--lib", "--tests", "-p", "macros"],
        &["-p", "gated"][..] => vec!["--lib", "--tests", "-p", "gated"],
        &["-p", "libonly", "--package", "binonly", "-p", "demo", "--", "-D", "warnings"][..] => vec!["--lib", "--bins", "--example", "showcase", "--tests", "-p", "libonly", "--package", "binonly", "-p", "demo", "--", "-D", "warnings"],
        &["--package=binonly"][..] => vec!["--bins", "--tests", "--package=binonly"],
        &["-pbinonly"][..] => vec!["--bins", "--tests", "-pbinonly"],
        &["-p=binonly"][..] => vec!["--bins", "--tests", "-p=binonly"],
        &["-vp", "binonly"][..] => vec!["--bins", "--tests", "-vp", "binonly"],
        &["--packages", "x"][..] => vec!["--lib", "--bins", "--tests", "--packages", "x"],
        &["--", "-D", "warnings"][..] => vec!["--lib", "--bins", "--tests", "--", "-D", "warnings"],
        &[][..] => vec!["--lib", "--bins", "--tests"],
        &["--workspace", "-p", "libonly"][..] => vec!["--lib", "--bins", "--example", "showcase", "--bench", "speed", "--tests", "--workspace", "-p", "libonly"],
        &["--", "-p", "benched"][..] => vec!["--lib", "--bins", "--tests", "--", "-p", "benched"],
        &["--", "--workspace"][..] => vec!["--lib", "--bins", "--tests", "--", "--workspace"],
    ]
);

const UNRESOLVED: &str = "against the cargo metadata: it takes workspace package names, not globs, versions or URLs";
const REFUSED: &str = "it chooses the targets from the cargo metadata of the workspace it runs in";
const NOTHING: &str = "the clippy scope has no target to check, so clippy would pass by checking nothing";
const NO_PACKAGES: &str = r#"{ "packages": [], "workspace_default_members": [] }"#;
const NO_DEFAULTS: &str =
    r#"{ "packages": [{ "id": "a", "name": "a", "targets": [{ "name": "a", "kind": ["lib"] }] }] }"#;

laws::table!(
    clippy_refuses_a_package_spec_it_cannot_resolve_naming_the_spec_and_never_runs,
    |scope: &&[&str]| Clippy::refused(scope),
    [
        &["-p", "unmatched"][..] => vec![format!("clippy-targets cannot resolve the package spec `unmatched` {UNRESOLVED}")],
        &["-p", "lib*"][..] => vec![format!("clippy-targets cannot resolve the package spec `lib*` {UNRESOLVED}")],
        &["-p", "libonly@0.1.0"][..] => vec![format!("clippy-targets cannot resolve the package spec `libonly@0.1.0` {UNRESOLVED}")],
        &["--package=libonly@0.1.0"][..] => vec![format!("clippy-targets cannot resolve the package spec `libonly@0.1.0` {UNRESOLVED}")],
        &["-plib?nly"][..] => vec![format!("clippy-targets cannot resolve the package spec `lib?nly` {UNRESOLVED}")],
        &["-p", "path+file:///w/libonly#0.1.0"][..] => vec![format!("clippy-targets cannot resolve the package spec `path+file:///w/libonly#0.1.0` {UNRESOLVED}")],
        &["-p", "libonly", "-p", "libonly:0.1.0"][..] => vec![format!("clippy-targets cannot resolve the package spec `libonly:0.1.0` {UNRESOLVED}")],
        &["--workspace", "-p", "unmatched"][..] => vec![format!("clippy-targets cannot resolve the package spec `unmatched` {UNRESOLVED}")],
        &["--package="][..] => vec![format!("clippy-targets cannot resolve the package spec `` {UNRESOLVED}")],
    ]
);

laws::table!(
    clippy_refuses_the_flags_that_would_choose_targets_or_another_workspace,
    |scope: &&[&str]| Clippy::refused(scope),
    [
        &["--workspace", "--exclude", "libonly"][..] => vec![format!("clippy-targets refuses `--exclude`: {REFUSED}")],
        &["--workspace", "--exclude=libonly"][..] => vec![format!("clippy-targets refuses `--exclude=libonly`: {REFUSED}")],
        &["--manifest-path", "w/Cargo.toml"][..] => vec![format!("clippy-targets refuses `--manifest-path`: {REFUSED}")],
        &["--workspace", "--all-targets"][..] => vec![format!("clippy-targets refuses `--all-targets`: {REFUSED}")],
        &["-p", "benched", "--benches"][..] => vec![format!("clippy-targets refuses `--benches`: {REFUSED}")],
        &["--bench=speed"][..] => vec![format!("clippy-targets refuses `--bench=speed`: {REFUSED}")],
        &["--lib"][..] => vec![format!("clippy-targets refuses `--lib`: {REFUSED}")],
        &["--tests"][..] => vec![format!("clippy-targets refuses `--tests`: {REFUSED}")],
    ]
);

laws::table!(
    clippy_refuses_a_package_flag_without_a_spec,
    |scope: &&[&str]| Clippy::refused(scope).concat().contains("usage: ci "),
    [
        &["-p"][..] => true,
        &["--package"][..] => true,
        &["-p", "--", "-D", "warnings"][..] => true,
    ]
);

laws::table!(
    clippy_refuses_a_scope_with_no_target_rather_than_checking_nothing,
    |(metadata, scope): &(&str, &[&str])| Clippy::refused_on(metadata, scope),
    [
        (NO_PACKAGES, &[][..]) => vec![NOTHING.to_owned()],
        (NO_PACKAGES, &["--workspace"][..]) => vec![NOTHING.to_owned()],
        (NO_DEFAULTS, &[][..]) => vec![NOTHING.to_owned()],
    ]
);

#[test]
fn clippy_never_names_a_kind_flag_that_puts_libraries_in_test_mode_and_names_each_bench_and_example() {
    let without = Clippy::invoked(&["-p", "libonly", "-p", "binonly", "-p", "demo", "-p", "gated"]);
    let with = Clippy::invoked(&["-p", "libonly", "-p", "benched"]);
    let whole = Clippy::invoked(&["--workspace"]);
    assert!(without.iter().all(|flag| flag != "--bench"));
    assert!(with.contains(&"--bench".to_owned()));
    assert!(
        without
            .iter()
            .chain(&with)
            .chain(&whole)
            .all(|flag| ["--all-targets", "--test", "--benches", "--examples"].iter().all(|named| flag != named))
    );
}

#[test]
fn clippy_is_timed_under_its_stage_and_fails_with_its_code() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Script { metadata: CLIPPY_FIXTURE.to_owned(), exit: Exit::Failure(FAILED_WITH), ..Script::new() };
    let answer = scratch.answer(&fake, &["clippy-targets", &scratch.file("m.toml"), "clippy", "-p", "binonly"]);
    assert_eq!(answer.exit, ExitCode::from(FAILED_WITH.get()));
    assert_eq!(Text::lines(&scratch.read("m.toml")), ["clippy = 1.50"]);
    assert_eq!(
        *fake.calls.borrow(),
        [
            Invocation::new("cargo", ["metadata", "--locked", "--no-deps", "--format-version", "1"]),
            Invocation::new("cargo", ["clippy", "--locked", "--bins", "--tests", "-p", "binonly"]),
        ]
    );
    Ok(())
}

#[test]
fn metadata_that_cannot_be_read_stops_clippy_before_it_runs() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Script { metadata: "not json".to_owned(), ..Script::new() };
    let answer = scratch.answer(&fake, &["clippy-targets", &scratch.file("m.toml"), "clippy", "--workspace"]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("cannot read the cargo metadata: "));
    assert_eq!(fake.calls.borrow().len(), 1);
    assert_eq!(scratch.read("m.toml"), "");
    Ok(())
}

#[test]
fn metadata_that_cannot_be_asked_for_stops_clippy_before_it_runs() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let answer = scratch.answer(&Script::refusing("cargo"), &["clippy-targets", &scratch.file("m.toml"), "clippy"]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert_eq!(Text::lines(&answer.err), ["cannot run cargo: boom"]);
    assert_eq!(scratch.read("m.toml"), "");
    Ok(())
}

#[test]
fn clippy_targets_needs_a_measured_file_and_a_stage() {
    assert!(Answer::refused(&["clippy-targets"]));
    assert!(Answer::refused(&["clippy-targets", "m.toml"]));
}

#[test]
fn a_timed_stage_runs_the_program_with_its_arguments() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Script::new();
    let answer =
        scratch.answer(&fake, &["timed", &scratch.file("m.toml"), "clippy", "cargo", "clippy", "--all-targets"]);
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: Vec::<&str>::new(), err: Vec::<&str>::new() });
    assert_eq!(*fake.calls.borrow(), [Invocation::new("cargo", ["clippy", "--all-targets"])]);
    let calls = fake.calls.borrow();
    assert_eq!(
        calls.first().map(|call| call.arguments().to_vec()),
        Some(vec![OsString::from("clippy"), OsString::from("--all-targets")])
    );
    assert_eq!(Text::lines(&scratch.read("m.toml")), ["clippy = 1.50"]);
    Ok(())
}

#[test]
fn a_timed_stage_is_appended_to_what_is_measured() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("m.toml", "fmt = 0.10\n")?;
    scratch.answer(&Script::new(), &["timed", &scratch.file("m.toml"), "clippy", "cargo"]);
    scratch.answer(&Script::new(), &["timed", &scratch.file("m.toml"), "lint", "cargo"]);
    assert_eq!(Text::lines(&scratch.read("m.toml")), ["fmt = 0.10", "clippy = 1.50", "lint = 1.50"]);
    Ok(())
}

#[test]
fn a_failing_program_fails_the_step_with_its_code_and_is_still_measured() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Script { exit: Exit::Failure(FAILED_WITH), ..Script::new() };
    let answer = scratch.answer(&fake, &["timed", &scratch.file("m.toml"), "clippy", "cargo"]);
    assert_eq!(answer.exit, ExitCode::from(FAILED_WITH.get()));
    assert_eq!(Text::lines(&scratch.read("m.toml")), ["clippy = 1.50"]);
    Ok(())
}

#[test]
fn a_program_that_cannot_start_is_reported_and_not_measured() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let answer = scratch.answer(&Script::refusing("cargo"), &["timed", &scratch.file("m.toml"), "clippy", "cargo"]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert_eq!(Text::lines(&answer.err), ["cannot run cargo: boom"]);
    assert_eq!(scratch.read("m.toml"), "");
    Ok(())
}

#[test]
fn a_measured_file_that_cannot_be_written_is_reported() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("blocker", "")?;
    let answer = scratch.answer(&Script::new(), &["timed", &scratch.file("blocker/m.toml"), "clippy", "cargo"]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("cannot use "));
    assert!(answer.err.contains("blocker/m.toml"));
    Ok(())
}

#[test]
fn libtest_seconds_are_appended_under_the_stage_name() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("m.toml", "clippy = 1.00\n")?;
    scratch.write("out.txt", "test result: ok. finished in 0.25s\n")?;
    let answer = scratch
        .answer(&Script::new(), &["libtest-seconds", &scratch.file("m.toml"), "test-run", &scratch.file("out.txt")]);
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: Vec::<&str>::new(), err: Vec::<&str>::new() });
    assert_eq!(Text::lines(&scratch.read("m.toml")), ["clippy = 1.00", "test-run = 0.25"]);
    Ok(())
}

#[test]
fn test_output_that_cannot_be_read_is_reported_and_nothing_is_measured() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let answer = scratch
        .answer(&Script::new(), &["libtest-seconds", &scratch.file("m.toml"), "test-run", &scratch.file("absent.txt")]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("absent.txt"));
    assert_eq!(scratch.read("m.toml"), "");
    Ok(())
}

#[test]
fn test_output_without_a_result_line_is_a_failure_and_nothing_is_measured() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("out.txt", "no result lines here\n")?;
    let answer = scratch
        .answer(&Script::new(), &["libtest-seconds", &scratch.file("m.toml"), "test-run", &scratch.file("out.txt")]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("no libtest result line"));
    assert_eq!(scratch.read("m.toml"), "");
    Ok(())
}

#[test]
fn a_result_line_libtest_could_not_have_written_is_a_failure_and_nothing_is_measured() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write(
        "out.txt",
        "test result: ok. 2 passed; finished in 1.00s\ntest result: ok. 1 passed; finished in soon\n",
    )?;
    let answer = scratch
        .answer(&Script::new(), &["libtest-seconds", &scratch.file("m.toml"), "test-run", &scratch.file("out.txt")]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("out.txt"));
    assert!(answer.err.contains("\"test result: ok. 1 passed; finished in soon\" in "));
    assert!(answer.err.contains("opens like a libtest result line but does not read as one"));
    assert_eq!(scratch.read("m.toml"), "");
    Ok(())
}

#[test]
fn off_a_pull_request_the_scope_is_the_whole_workspace() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Script::new();
    let answer = scratch.answer(&fake, &Reach::on(&scratch, "push"));
    assert_eq!(
        answer.lines(),
        Printed { exit: ExitCode::SUCCESS, out: vec!["BUILD_SCOPE=--workspace"], err: Vec::<&str>::new() }
    );
    assert_eq!(Text::lines(&scratch.read("env")), ["BUILD_SCOPE=--workspace"]);
    assert_eq!(Text::lines(&scratch.read("sum")), ["BUILD_SCOPE=--workspace"]);
    assert_eq!(
        *fake.calls.borrow(),
        [Invocation::new("cargo", ["metadata", "--locked", "--no-deps", "--format-version", "1"])]
    );
    assert_eq!(scratch.read("metadata.json"), METADATA);
    Ok(())
}

#[test]
fn on_a_pull_request_the_changed_files_are_asked_of_git() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Script { changed: "crates/leaf/src/lib.rs\n".to_owned(), ..Script::new() };
    let answer = scratch.answer(&fake, &Reach::on(&scratch, "pull_request"));
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: LEAF_REPORT.to_vec(), err: Vec::<&str>::new() });
    assert_eq!(Text::lines(&scratch.read("env")), LEAF_REPORT);
    assert_eq!(Text::lines(&scratch.read("sum")), LEAF_REPORT);
    assert_eq!(
        *fake.calls.borrow(),
        [
            Invocation::new("cargo", ["metadata", "--locked", "--no-deps", "--format-version", "1"]),
            Invocation::new(
                "git",
                ["-c", "core.quotePath=false", "diff", "--name-only", "--no-renames", "HEAD^1", "HEAD"]
            ),
        ]
    );
    assert_eq!(scratch.read("metadata.json"), METADATA);
    Ok(())
}

#[test]
fn a_history_line_names_the_revision_and_the_reach_and_is_appended() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("metadata.json", METADATA)?;
    scratch.write("history.tsv", "older\n")?;
    let fake = Script { changed: "crates/leaf/src/lib.rs\n".to_owned(), ..Script::new() };
    let arguments = ["history", &scratch.file("metadata.json"), &scratch.file("history.tsv"), "abc123"];
    let answer = scratch.answer(&fake, &arguments);
    assert_eq!(answer.exit, ExitCode::SUCCESS);
    assert_eq!(Text::lines(&answer.out), [HISTORY_LINE]);
    scratch.answer(&fake, &arguments);
    assert_eq!(Text::lines(&scratch.read("history.tsv")), ["older", HISTORY_LINE, HISTORY_LINE]);
    Ok(())
}

#[test]
fn a_measured_history_line_names_the_revision_and_every_stage_and_is_appended() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("m.toml", "cold-clippy = 52.3\ncold-test-compile = 80\n")?;
    scratch.write("history.tsv", "older\n")?;
    let arguments = ["measured-history", &scratch.file("m.toml"), &scratch.file("history.tsv"), "abc123"];
    let answer = scratch.answer(&Script::new(), &arguments);
    let line = "abc123\tcold-clippy = 52.3\tcold-test-compile = 80";
    assert_eq!((answer.exit, Text::lines(&answer.out)), (ExitCode::SUCCESS, vec![line.to_owned()]));
    assert_eq!(Text::lines(&scratch.read("history.tsv")), ["older", line]);
    let missing = ["measured-history", &scratch.file("none.toml"), &scratch.file("history.tsv"), "abc123"];
    assert_eq!(scratch.answer(&Script::new(), &missing).exit, ExitCode::FAILURE);
    Ok(())
}

#[test]
fn a_cold_size_sums_every_nested_file_and_rounds_up_to_kibibytes() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.placed(&scratch.path().join("cold/a.bin"), &"a".repeat(1000))?;
    scratch.placed(&scratch.path().join("cold/deep/er/b.bin"), &"b".repeat(1500))?;
    let arguments = ["cold-size", &scratch.file("cold"), &scratch.file("cold/size.json")];
    let answer = scratch.answer(&Script::new(), &arguments);
    assert_eq!(answer.exit, ExitCode::SUCCESS);
    assert_eq!(Text::lines(&scratch.read("cold/size.json")), ["{\"kilobytes\": 3}"]);
    Ok(())
}

#[test]
fn a_cold_size_of_an_exact_kibibyte_is_not_rounded_up() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.placed(&scratch.path().join("cold/a.bin"), &"a".repeat(2048))?;
    let arguments = ["cold-size", &scratch.file("cold"), &scratch.file("size.json")];
    scratch.answer(&Script::new(), &arguments);
    assert_eq!(Text::lines(&scratch.read("size.json")), ["{\"kilobytes\": 2}"]);
    Ok(())
}

#[test]
fn an_empty_cold_directory_is_zero_kilobytes() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.directory("cold")?;
    let arguments = ["cold-size", &scratch.file("cold"), &scratch.file("size.json")];
    assert_eq!(scratch.answer(&Script::new(), &arguments).exit, ExitCode::SUCCESS);
    assert_eq!(Text::lines(&scratch.read("size.json")), ["{\"kilobytes\": 0}"]);
    Ok(())
}

#[test]
fn an_unreadable_cold_directory_is_reported_and_nothing_is_written() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let arguments = ["cold-size", &scratch.file("none"), &scratch.file("size.json")];
    let answer = scratch.answer(&Script::new(), &arguments);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("cannot use "));
    assert_eq!(scratch.kind_of("size.json"), Kind::Missing);
    Ok(())
}

const LOWERED_RECORD: [&str; 2] = ["cold-clippy = 48.5", "cold-test-compile = 80"];

#[test]
fn a_ratchet_record_is_lowered_to_a_run_at_or_below_it_and_never_raised() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("record.toml", "cold-clippy = 50\ncold-test-compile = 80\n")?;
    scratch.write("lower.toml", "cold-clippy = 48.5\ncold-test-compile = 80\n")?;
    scratch.write("above.toml", "cold-clippy = 40\ncold-test-compile = 81\n")?;
    let lower = |measured: &str| {
        scratch.answer(&Script::new(), &["ratchet-lower", &scratch.file("record.toml"), &scratch.file(measured)])
    };
    let lowered = lower("lower.toml");
    assert_eq!(lowered.exit, ExitCode::SUCCESS);
    assert_eq!(Text::lines(&scratch.read("record.toml")), LOWERED_RECORD);
    let above = lower("above.toml");
    assert_eq!(above.exit, ExitCode::FAILURE);
    assert!(above.err.contains("cold-test-compile took 81s"), "{}", above.err);
    assert_eq!(Text::lines(&scratch.read("record.toml")), LOWERED_RECORD);
    Ok(())
}

#[test]
fn a_history_needs_git_and_readable_metadata_and_a_writable_file() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("metadata.json", METADATA)?;
    let named = |file: &str| ["history".to_owned(), scratch.file("metadata.json"), scratch.file(file), "r".to_owned()];
    let git = scratch.answer(&Script::refusing("git"), &named("history.tsv"));
    assert_eq!((git.exit, Text::lines(&git.err)), (ExitCode::FAILURE, vec!["cannot run git: boom".to_owned()]));
    scratch.write("blocker", "")?;
    assert_eq!(scratch.answer(&Script::new(), &named("blocker/history.tsv")).exit, ExitCode::FAILURE);
    let broken = ["history".to_owned(), scratch.file("none.json"), scratch.file("history.tsv"), "r".to_owned()];
    assert!(scratch.answer(&Script::new(), &broken).err.contains("reach failed: cannot read"));
    assert_eq!(scratch.read("history.tsv"), "");
    Ok(())
}

#[test]
fn a_report_is_appended_to_the_summary_under_its_name_or_said_to_be_missing() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("report.txt", "found\n")?;
    scratch.write("summary.md", "# Summary\n")?;
    let arguments = |report: &str| ["summarize".to_owned(), scratch.file(report), scratch.file("summary.md")];
    assert_eq!(scratch.answer(&Script::new(), &arguments("report.txt")).exit, ExitCode::SUCCESS);
    assert_eq!(scratch.answer(&Script::new(), &arguments("absent.txt")).exit, ExitCode::SUCCESS);
    let (found, absent) = (format!("{}:", scratch.file("report.txt")), format!("{}:", scratch.file("absent.txt")));
    assert_eq!(Text::lines(&scratch.read("summary.md")), ["# Summary", &found, "found", &absent, "not produced"]);
    Ok(())
}

struct Timings {
    answer: Answer,
    stored: String,
}

struct Page;

impl Page {
    fn with(units: &str) -> String {
        format!("<html>\nconst OTHER = [];\nconst UNIT_DATA = [{units}\n];\nconst CONCURRENCY_DATA = [];\n</html>\n")
    }

    fn unit(name: &str, duration: &str) -> String {
        format!("{{\"i\": 0, \"name\": \"{name}\", \"mode\": \"todo\", \"duration\": {duration}, \"sections\": null}}")
    }

    fn listing(units: &str) -> String {
        format!("<html>\nconst UNIT_DATA = {units};\n</html>\n")
    }

    fn of(name: &str, version: &str, target: &str, duration: &str) -> String {
        format!(
            "{{\"i\": 0, \"name\": \"{name}\", \"version\": \"{version}\", \"mode\": \"todo\", \"target\": \"{}\", \"duration\": {duration}}}",
            target.escape_default()
        )
    }

    fn timed(page: &str) -> Result<Timings, Box<dyn Error>> {
        let scratch = Scratch::new()?;
        scratch.write("page.html", page)?;
        let answer =
            scratch.answer(&Script::new(), &["compile-times", &scratch.file("page.html"), &scratch.file("report.txt")]);
        Ok(Timings { answer, stored: scratch.read("report.txt") })
    }
}

#[test]
fn compile_time_is_summed_per_crate_longest_first_and_stored() -> Result<(), Box<dyn Error>> {
    let page = Page::with(&[Page::unit("aaa", "0.25"), Page::unit("zzz", "1.5"), Page::unit("zzz", "0.5")].join(","));
    let Timings { answer, stored } = Page::timed(&page)?;
    let expected = vec!["compile time per crate, seconds, longest first, 2.25 in all:", "2.00 zzz", "0.25 aaa"];
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: expected.clone(), err: Vec::<&str>::new() });
    assert_eq!(Text::lines(&stored), expected);
    Ok(())
}

#[test]
fn only_the_thirty_longest_crates_are_listed_but_all_are_counted() -> Result<(), Box<dyn Error>> {
    let units: Vec<String> = (0..CRATES_GENERATED).map(|at| Page::unit(&format!("crate{at:02}"), "1.0")).collect();
    let answer = Page::timed(&Page::with(&units.join(",")))?.answer;
    assert_eq!(answer.out.matches(char::from(10)).count(), LINES_PRINTED);
    assert!(answer.out.contains("compile time per crate, seconds, longest first, 31.00 in all:\n"));
    Ok(())
}

#[test]
fn a_page_without_readable_unit_data_is_refused() -> Result<(), Box<dyn Error>> {
    let pages = ["<html></html>", "const UNIT_DATA = nope;", "const UNIT_DATA = [{\"name\": 3}];"];
    let refused = pages
        .iter()
        .map(|page| {
            Page::timed(page).map(|Timings { answer, stored }| {
                (*page, answer.exit, answer.err.contains("no unit timings in"), answer.out, stored)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(refused, pages.map(|page| (page, ExitCode::FAILURE, true, String::new(), String::new())));
    let scratch = Scratch::new()?;
    let missing =
        scratch.answer(&Script::new(), &["compile-times", &scratch.file("none.html"), &scratch.file("report.txt")]);
    assert_eq!(missing.exit, ExitCode::FAILURE);
    Ok(())
}

struct Graph {
    answer: Answer,
    measured: String,
    report: String,
    asked: Vec<Invocation>,
}

impl Graph {
    fn metadata_for(host: &str) -> Invocation {
        Invocation::new("cargo", ["metadata", "--locked", "--format-version", "1", "--filter-platform", host])
    }

    fn hosted(script: Script) -> Script {
        let answers = script.answers.into_iter().chain([Reply::to("rustc --print", &format!("{HOST}\n"))]).collect();
        Script { answers, ..script }
    }

    fn measured(page: &str, metadata: &str) -> Result<Graph, Box<dyn Error>> {
        Graph::answered(page, &Graph::hosted(Script { metadata: metadata.to_owned(), ..Script::new() }))
    }

    fn answered(page: &str, fake: &Script) -> Result<Graph, Box<dyn Error>> {
        let scratch = Scratch::new()?;
        scratch.write("page.html", page)?;
        scratch.write("m.toml", "clippy = 50.80\n")?;
        let arguments = ["graph-path", &scratch.file("page.html"), &scratch.file("m.toml"), &scratch.file("path.txt")];
        let answer = scratch.answer(fake, &arguments);
        let asked = fake.calls.borrow().clone();
        Ok(Graph { answer, measured: scratch.read("m.toml"), report: scratch.read("path.txt"), asked })
    }
}

#[test]
fn the_graph_path_is_the_longest_chain_of_uncacheable_units_and_is_measured_beside_the_wall()
-> Result<(), Box<dyn Error>> {
    let Graph { answer, measured, report, asked } = Graph::measured(&Page::listing(TIMINGS_UNITS), TIMINGS_METADATA)?;
    assert_eq!(asked, [Invocation::new("rustc", ["--print", "host-tuple"]), Graph::metadata_for(HOST)]);
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: GRAPH_PATH.to_vec(), err: Vec::<&str>::new() });
    assert_eq!(Text::lines(&report), GRAPH_PATH);
    assert_eq!(Text::lines(&measured), ["clippy = 50.80", "graph-path = 2.50"]);
    Ok(())
}

#[test]
fn cacheable_external_units_weigh_nothing_however_long_they_took() -> Result<(), Box<dyn Error>> {
    let page = Page::with(
        &[
            Page::of("heavy", "1.0.0", " (check)", "9.0"),
            Page::of("heavy", "1.0.0", "", "5.0"),
            Page::of("sys", "0.3.0", " (check)", "1.0"),
        ]
        .join(","),
    );
    let Graph { answer, measured, report, .. } = Graph::measured(&page, TIMINGS_METADATA)?;
    assert_eq!(answer.exit, ExitCode::SUCCESS);
    assert_eq!(Text::lines(&measured), ["clippy = 50.80", "graph-path = 0.00"]);
    assert_eq!(
        Text::lines(&report).last().map(String::as_str),
        Some("graph-work 0.00 seconds, the sum of the same units")
    );
    Ok(())
}

#[test]
fn a_proc_macro_host_build_and_a_build_script_count_even_outside_the_workspace() -> Result<(), Box<dyn Error>> {
    let page = Page::with(
        &[
            Page::of("ext-macro", "2.0.0", "", "0.4"),
            Page::of("sys", "0.3.0", " build-script", "0.1"),
            Page::of("sys", "0.3.0", " build-script (run)", "0.2"),
            Page::of("sys", "0.3.0", " (check)", "1.0"),
        ]
        .join(","),
    );
    let Graph { answer, measured, report, .. } = Graph::measured(&page, TIMINGS_METADATA)?;
    let expected = [
        "graph-path 0.40 seconds, the longest chain of clippy units, counting only units sccache never caches:",
        "0.40 ext-macro (build)",
        "graph-work 0.70 seconds, the sum of the same units",
    ];
    assert_eq!(Text::lines(&answer.out), expected);
    assert_eq!(Text::lines(&measured), ["clippy = 50.80", "graph-path = 0.40"]);
    assert_eq!(
        Text::lines(&report).last().map(String::as_str),
        Some("graph-work 0.70 seconds, the sum of the same units")
    );
    Ok(())
}

#[test]
fn a_unit_the_page_does_not_time_weighs_nothing_but_still_links_its_neighbours() -> Result<(), Box<dyn Error>> {
    let page = Page::with(
        &[
            Page::of("derive-macro", "0.1.0", "", "0.3"),
            Page::of("app", "0.1.0", r#" behaviour "test" (check-test)"#, "0.8"),
        ]
        .join(","),
    );
    let Graph { answer, measured, report, .. } = Graph::measured(&page, TIMINGS_METADATA)?;
    let expected = [
        "graph-path 1.10 seconds, the longest chain of clippy units, counting only units sccache never caches:",
        "0.30 derive-macro (build)",
        r#"0.80 app behaviour "test" (check-test)"#,
        "graph-work 1.10 seconds, the sum of the same units",
    ];
    assert_eq!(Text::lines(&answer.out), expected);
    assert_eq!(Text::lines(&measured), ["clippy = 50.80", "graph-path = 1.10"]);
    assert_eq!(
        Text::lines(&report).last().map(String::as_str),
        Some("graph-work 1.10 seconds, the sum of the same units")
    );
    Ok(())
}

#[test]
fn a_unit_label_cargo_never_writes_refuses_the_page_and_nothing_is_measured() -> Result<(), Box<dyn Error>> {
    let labels = [" app bin (check)", " (check) ", " (build)", " app \"bin\"", " build-script (check)"];
    let refused = labels
        .iter()
        .map(|label| {
            Graph::measured(&Page::with(&Page::of("app", "0.1.0", label, "1.0")), TIMINGS_METADATA)
                .map(|graph| (graph.answer.exit, graph.answer.err.contains("no unit timings in"), graph.measured))
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(refused, vec![(ExitCode::FAILURE, true, "clippy = 50.80\n".to_owned()); labels.len()]);
    Ok(())
}

#[test]
fn two_check_variants_of_one_crate_count_the_longer_on_the_path_and_both_in_the_work() -> Result<(), Box<dyn Error>> {
    let page = Page::with(
        &[Page::of("core", "0.1.0", " (check)", "0.5"), Page::of("core", "0.1.0", " (check)", "0.7")].join(","),
    );
    let Graph { answer, measured, report, .. } = Graph::measured(&page, TIMINGS_METADATA)?;
    assert_eq!(Text::lines(&answer.out).get(1).map(String::as_str), Some("0.70 core (check)"));
    assert_eq!(Text::lines(&measured), ["clippy = 50.80", "graph-path = 0.70"]);
    assert_eq!(
        Text::lines(&report).last().map(String::as_str),
        Some("graph-work 1.20 seconds, the sum of the same units")
    );
    Ok(())
}

#[test]
fn a_graph_path_needs_unit_timings_and_metadata_cargo_can_give() -> Result<(), Box<dyn Error>> {
    let unreadable = Graph::measured(&Page::listing(TIMINGS_UNITS), "not json")?;
    assert_eq!(unreadable.answer.exit, ExitCode::FAILURE);
    assert!(unreadable.answer.err.contains("cannot read the cargo metadata: "));
    assert_eq!(Text::lines(&unreadable.measured), ["clippy = 50.80"]);
    assert_eq!(unreadable.report, "");
    let untimed = Graph::measured("<html></html>", TIMINGS_METADATA)?;
    assert_eq!(untimed.answer.exit, ExitCode::FAILURE);
    assert!(untimed.answer.err.contains("no unit timings in"));
    assert_eq!(untimed.asked, Vec::<Invocation>::new());
    let scratch = Scratch::new()?;
    scratch.write("page.html", &Page::listing(TIMINGS_UNITS))?;
    let refused = scratch.answer(
        &Graph::hosted(Script::refusing("cargo")),
        &["graph-path", &scratch.file("page.html"), &scratch.file("m.toml"), &scratch.file("path.txt")],
    );
    assert_eq!(
        (refused.exit, Text::lines(&refused.err)),
        (ExitCode::FAILURE, vec!["cannot run cargo: boom".to_owned()])
    );
    assert_eq!(scratch.read("m.toml"), "");
    Ok(())
}

#[test]
fn the_graph_follows_only_the_dependencies_cargo_resolves_for_the_host_platform() -> Result<(), Box<dyn Error>> {
    let page = Page::with(
        &[
            Page::of("slowmacro", "0.1.0", "", "1.0"),
            Page::of("mid", "0.1.0", " (check)", "1.0"),
            Page::of("app", "0.1.0", " (check)", "1.0"),
            Page::of("user", "0.1.0", " (check)", "0.5"),
        ]
        .join(","),
    );
    let fake = Script {
        metadata: EVERY_PLATFORM.to_owned(),
        exact: vec![ExactReply { asked: Graph::metadata_for(HOST), text: HOST_PLATFORM.to_owned() }],
        ..Graph::hosted(Script::new())
    };
    let Graph { answer, measured, report, .. } = Graph::answered(&page, &fake)?;
    let expected = [
        "graph-path 2.00 seconds, the longest chain of clippy units, counting only units sccache never caches:",
        "1.00 mid (check)",
        "1.00 app (check)",
        "graph-work 3.50 seconds, the sum of the same units",
    ];
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: expected.to_vec(), err: Vec::<&str>::new() });
    assert_eq!(Text::lines(&measured), ["clippy = 50.80", "graph-path = 2.00"]);
    assert_eq!(
        Text::lines(&report).last().map(String::as_str),
        Some("graph-work 3.50 seconds, the sum of the same units")
    );
    Ok(())
}

#[test]
fn a_page_that_records_a_failed_build_is_skipped_and_nothing_is_measured() -> Result<(), Box<dyn Error>> {
    let failed = format!(
        "<table>\n<tr><td class=\"error-text\">Error:</td><td>1 job failed</td></tr>\n</table>\n{}",
        Page::listing(TIMINGS_UNITS)
    );
    let Graph { answer, measured, report, asked } = Graph::measured(&failed, TIMINGS_METADATA)?;
    let skipped = "graph-path skipped: the timings page records a failed build, so it times only part of the clippy units and nothing is measured";
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: vec![skipped], err: Vec::<&str>::new() });
    assert_eq!(Text::lines(&report), [skipped]);
    assert_eq!(Text::lines(&measured), ["clippy = 50.80"]);
    assert_eq!(asked, Vec::<Invocation>::new());
    Ok(())
}

#[test]
fn a_graph_path_needs_rustc_to_name_exactly_one_host() -> Result<(), Box<dyn Error>> {
    let named = ["", "x86_64-unknown-linux-gnu\naarch64-apple-darwin\n"].map(|host| {
        let fake = Script::answering(vec![Reply::to("rustc --print", host)]);
        Graph::answered(&Page::listing(TIMINGS_UNITS), &fake).map(|graph| {
            (
                graph.answer.exit,
                graph.answer.err.contains("not the one host platform"),
                graph.measured,
                graph.asked.len(),
            )
        })
    });
    let refused = named.into_iter().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(refused, vec![(ExitCode::FAILURE, true, "clippy = 50.80\n".to_owned(), 1); 2]);
    Ok(())
}

#[test]
fn a_program_the_scope_needs_that_cannot_start_is_reported() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let cargo = scratch.answer(&Script::refusing("cargo"), &Reach::on(&scratch, "push"));
    assert_eq!((cargo.exit, Text::lines(&cargo.err)), (ExitCode::FAILURE, vec!["cannot run cargo: boom".to_owned()]));
    let git = scratch.answer(&Script::refusing("git"), &Reach::on(&scratch, "pull_request"));
    assert_eq!((git.exit, Text::lines(&git.err)), (ExitCode::FAILURE, vec!["cannot run git: boom".to_owned()]));
    assert_eq!(git.out, "");
    Ok(())
}

#[test]
fn metadata_reach_cannot_use_is_reported_with_its_reason() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Script { metadata: "not json".to_owned(), ..Script::new() };
    let answer = scratch.answer(&fake, &Reach::on(&scratch, "pull_request"));
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("reach failed: cannot parse cargo metadata"));
    assert_eq!(scratch.read("env"), "");
    assert_eq!(answer.out, "");
    Ok(())
}

#[test]
fn metadata_that_cannot_be_stored_is_reported() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("blocker", "")?;
    let answer =
        scratch.answer(&Script::new(), &["reach", &scratch.file("blocker/metadata.json"), "push", "", "e", "s"]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("blocker/metadata.json"));
    Ok(())
}

laws::table!(
    unusable_input_is_refused_with_the_usage,
    Answer::refused,
    [
        [] => true,
        ["other"] => true,
        ["timed"] => true,
        ["timed", "m.toml"] => true,
        ["timed", "m.toml", "clippy"] => true,
        ["libtest-seconds"] => true,
        ["libtest-seconds", "m.toml", "test-run"] => true,
        ["libtest-seconds", "m.toml", "test-run", "out.txt", "extra"] => true,
        ["reach"] => true,
        ["reach", "metadata.json"] => true,
        ["reach", "metadata.json", "push"] => true,
        ["reach", "metadata.json", "push", "main"] => true,
        ["reach", "metadata.json", "push", "main", "env"] => true,
        ["reach", "metadata.json", "push", "main", "env", "sum", "extra"] => true,
        ["stage-issues"] => true,
        ["stage-issues", "limits.toml"] => true,
        ["compile-trend"] => true,
        ["compile-trend", "today"] => true,
        ["compile-trend", "today", "runs", "extra"] => true,
        ["compile-times"] => true,
        ["compile-times", "page.html"] => true,
        ["compile-times", "page.html", "report.txt", "extra"] => true,
        ["graph-path"] => true,
        ["graph-path", "page.html"] => true,
        ["graph-path", "page.html", "m.toml"] => true,
        ["graph-path", "page.html", "m.toml", "path.txt", "extra"] => true,
        ["history"] => true,
        ["history", "metadata.json"] => true,
        ["history", "metadata.json", "history.tsv"] => true,
        ["history", "metadata.json", "history.tsv", "abc", "extra"] => true,
        ["measured-history", "m.toml", "history.tsv"] => true,
        ["ratchet-lower", "record.toml"] => true,
        ["ratchet-lower", "record.toml", "m.toml", "extra"] => true,
        ["logged"] => true,
        ["logged", "log.txt"] => true,
        ["tools"] => true,
        ["tools", "built"] => true,
        ["tools", "built", "to", "extra"] => true,
        ["tools-key"] => true,
        ["tools-key", "metadata.json"] => true,
        ["tools-key", "metadata.json", "out", "extra"] => true,
        ["mutants-shard"] => true,
        ["mutants-shard", "0/8"] => true,
        ["mutants-shard", "0/8", "out", "extra"] => true,
        ["mutants-shard", "8/8", "out"] => true,
        ["mutants-shard", "0/0", "out"] => true,
        ["mutants-shard", "3", "out"] => true,
        ["mutants-changed"] => true,
        ["mutants-changed", "diff"] => true,
        ["mutants-changed", "diff", "out"] => true,
        ["mutants-changed", "diff", "out", "8/8", "-p"] => true,
        ["mutants-changed", "diff", "out", "-1/8", "-p"] => true,
        ["mutants-changed", "diff", "out", "--test-package=leaf"] => true,
        ["wall-times"] => true,
        ["wall-times", "repo", "1"] => true,
        ["wall-times", "repo", "1", "job", "extra"] => true,
        ["show"] => true,
        ["summarize"] => true,
        ["summarize", "report.txt"] => true,
        ["summarize", "report.txt", "summary.md", "extra"] => true,
        ["labels"] => true,
        ["labels", "main"] => true,
        ["labels", "main", "1"] => true,
        ["labels", "main", "1", "metadata.json", "extra"] => true,
        ["spec-line"] => true,
        ["spec-line", "1", "extra"] => true,
        ["shard-summary"] => true,
        ["shard-summary", "shards"] => true,
        ["matrix"] => true,
        ["matrix", "shards", "sources", "out"] => true,
        ["matrix", "shards", "sources", "out", "sum", "extra"] => true,
    ]
);

#[test]
fn a_reach_report_is_appended_to_the_environment_and_the_summary() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("env", "KEPT=1\n")?;
    scratch.write("sum", "# Summary\n")?;
    scratch.answer(&Script::new(), &Reach::on(&scratch, "push"));
    assert_eq!(Text::lines(&scratch.read("env")), ["KEPT=1", "BUILD_SCOPE=--workspace"]);
    assert_eq!(Text::lines(&scratch.read("sum")), ["# Summary", "BUILD_SCOPE=--workspace"]);
    Ok(())
}

#[test]
fn an_empty_scope_is_said_in_the_output_and_the_summary_but_not_put_in_the_environment() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let answer = scratch.answer(&Script::new(), &Reach::on(&scratch, "pull_request"));
    assert_eq!(Text::lines(&answer.out), EMPTY_REPORT_AND_NOTES);
    assert_eq!(Text::lines(&scratch.read("env")), EMPTY_REPORT);
    assert_eq!(Text::lines(&scratch.read("sum")), EMPTY_REPORT_AND_NOTES);
    assert_eq!([EMPTY_REPORT.as_slice(), EMPTY_NOTES.as_slice()].concat(), EMPTY_REPORT_AND_NOTES);
    Ok(())
}

#[test]
fn a_scope_with_crates_says_nothing_is_skipped() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Script { changed: "crates/leaf/src/lib.rs\n".to_owned(), ..Script::new() };
    assert_eq!(scratch.answer(&fake, &Reach::on(&scratch, "pull_request")).out.matches("skipped").count(), 0);
    Ok(())
}

#[test]
fn the_tools_are_built_and_copied_to_the_tools_directory() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.directory("built")?;
    ["reach", "lint", "ci"].iter().try_for_each(|name| scratch.write(&format!("built/{name}"), name))?;
    let fake = Script::new();
    let answer = scratch.answer(&fake, &["tools", &scratch.file("built"), &scratch.file("a/b/tools")]);
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: Vec::<&str>::new(), err: Vec::<&str>::new() });
    assert_eq!(*fake.calls.borrow(), [Invocation::new("cargo", [BUILD.as_slice(), &["--bin", "ci"]].concat())]);
    assert_eq!(["reach", "lint", "ci"].map(|name| scratch.read(&format!("a/b/tools/{name}"))), ["reach", "lint", "ci"]);
    Ok(())
}

#[test]
fn tools_that_failed_to_build_are_not_copied() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let answer = scratch.answer(&Script::exiting(101), &["tools", &scratch.file("built"), &scratch.file("tools")]);
    assert_eq!(answer.exit, ExitCode::from(101));
    assert_eq!(scratch.kind_of("tools"), Kind::Missing);
    Ok(())
}

#[test]
fn tools_that_were_not_built_are_reported() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let answer = scratch.answer(&Script::new(), &["tools", &scratch.file("built"), &scratch.file("tools")]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("built/reach"));
    Ok(())
}

#[test]
fn tools_cannot_go_where_a_file_is_and_cargo_must_start() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("blocker", "")?;
    let blocked = scratch.answer(&Script::new(), &["tools", &scratch.file("built"), &scratch.file("blocker/tools")]);
    assert!(blocked.err.contains("blocker/tools"));
    let refused =
        scratch.answer(&Script::refusing("cargo"), &["tools", &scratch.file("built"), &scratch.file("tools")]);
    assert_eq!(Text::lines(&refused.err), ["cannot run cargo: boom"]);
    Ok(())
}

const TOOLS_WORKSPACE: &str = r#"{"workspace_root": "/w", "packages": [
  {"name": "reach", "manifest_path": "/w/crates/reach/Cargo.toml", "dependencies": [{"name": "text-lines"}]},
  {"name": "lint", "manifest_path": "/w/crates/lint/Cargo.toml", "dependencies": [{"name": "lint-core"}]},
  {"name": "ci", "manifest_path": "/w/crates/ci/Cargo.toml", "dependencies": [{"name": "reach"}]},
  {"name": "text-lines", "manifest_path": "/w/crates/text-lines/Cargo.toml", "dependencies": []},
  {"name": "lint-core", "manifest_path": "/w/crates/lint-core/Cargo.toml", "dependencies": []},
  {"name": "gate-limits", "manifest_path": "/w/crates/gate-limits/Cargo.toml", "dependencies": [{"name": "ci"}]}
]}"#;
const TOOLS_SOURCES: [&str; 12] = [
    "sources<<TOOLS_SOURCES",
    "crates/ci/Cargo.toml",
    "crates/ci/src/**",
    "crates/lint/Cargo.toml",
    "crates/lint/src/**",
    "crates/lint-core/Cargo.toml",
    "crates/lint-core/src/**",
    "crates/reach/Cargo.toml",
    "crates/reach/src/**",
    "crates/text-lines/Cargo.toml",
    "crates/text-lines/src/**",
    "TOOLS_SOURCES",
];

#[test]
fn the_tools_key_lists_the_sources_of_the_tools_and_what_they_depend_on_and_no_others() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("out", "KEPT=1\n")?;
    let fake = Script { metadata: TOOLS_WORKSPACE.to_owned(), ..Script::new() };
    let answer = scratch.answer(&fake, &["tools-key", &scratch.file("m/metadata.json"), &scratch.file("out")]);
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: Vec::<&str>::new(), err: Vec::<&str>::new() });
    assert_eq!(Text::lines(&scratch.read("out")), [["KEPT=1"].as_slice(), TOOLS_SOURCES.as_slice()].concat());
    Ok(())
}

#[test]
fn a_tools_key_needs_metadata_cargo_can_give_and_reach_can_read() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let arguments = ["tools-key", &scratch.file("metadata.json"), &scratch.file("out")];
    assert_eq!(Text::lines(&scratch.answer(&Script::refusing("cargo"), &arguments).err), ["cannot run cargo: boom"]);
    let broken = Script { metadata: "not json".to_owned(), ..Script::new() };
    assert!(scratch.answer(&broken, &arguments).err.contains("reach failed: cannot parse cargo metadata"));
    assert_eq!(scratch.kind_of("out"), Kind::Missing);
    Ok(())
}

#[test]
fn a_logged_program_is_run_with_its_log_and_its_exit() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Script { logged_text: "seen\n".to_owned(), exit: Exit::Failure(FAILED_WITH), ..Script::new() };
    let answer = scratch.answer(&fake, &["logged", &scratch.file("logs/test.txt"), "cargo", "test", "-p", "leaf"]);
    assert_eq!(
        answer.lines(),
        Printed { exit: ExitCode::from(FAILED_WITH.get()), out: Vec::<&str>::new(), err: Vec::<&str>::new() }
    );
    assert_eq!(*fake.calls.borrow(), [Invocation::new("cargo", ["test", "-p", "leaf"])]);
    assert_eq!(*fake.logs.borrow(), [scratch.path().join("logs/test.txt")]);
    assert_eq!(Text::lines(&scratch.read("logs/test.txt")), ["seen"]);
    let refused = scratch.answer(&Script::refusing("cargo"), &["logged", &scratch.file("t.txt"), "cargo"]);
    assert_eq!(Text::lines(&refused.err), ["cannot run cargo: boom"]);
    Ok(())
}

struct Cargo;

impl Cargo {
    fn mutants(shard: &str, output: &str) -> Invocation {
        Invocation::new(
            "cargo",
            [
                "mutants",
                "--shard",
                shard,
                "--sharding",
                "round-robin",
                "--jobs",
                "2",
                "--colors",
                "never",
                "--output",
                output,
                "--",
                "--no-fail-fast",
            ],
        )
    }
}

#[derive(Debug)]
struct Code(u8);

struct Exits;

impl Exits {
    fn shard(code: &Code) -> ExitCode {
        Scratch::fresh().answer(&Script::exiting(code.0), &["mutants-shard", "3/8", "out"]).exit
    }

    fn changed(code: &Code) -> ExitCode {
        let scratch = Scratch::fresh();
        let arguments =
            ["mutants-changed", &scratch.file("d.diff"), &scratch.file("out"), "3/8", "--test-package=leaf"];
        scratch.answer(&Script::exiting(code.0), &arguments).exit
    }
}

laws::table!(
    a_shard_passes_when_mutants_survive_or_time_out_but_not_on_other_failures,
    Exits::shard,
    [
        Code(0) => ExitCode::SUCCESS,
        Code(1) => ExitCode::from(1),
        Code(2) => ExitCode::SUCCESS,
        Code(3) => ExitCode::SUCCESS,
        Code(4) => ExitCode::from(4),
        Code(101) => ExitCode::from(101),
    ]
);

laws::table!(
    changed_lines_pass_when_mutants_time_out_but_not_on_other_failures,
    Exits::changed,
    [
        Code(0) => ExitCode::SUCCESS,
        Code(1) => ExitCode::from(1),
        Code(2) => ExitCode::from(2),
        Code(3) => ExitCode::SUCCESS,
        Code(4) => ExitCode::from(4),
    ]
);

#[test]
fn a_shard_runs_cargo_mutants_for_its_slice() {
    let fake = Script::new();
    let answer = Scratch::fresh().answer(&fake, &["mutants-shard", "3/8", "/tmp/out"]);
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: Vec::<&str>::new(), err: Vec::<&str>::new() });
    assert_eq!(*fake.calls.borrow(), [Cargo::mutants("3/8", "/tmp/out")]);
    let refused = Scratch::fresh().answer(&Script::refusing("cargo"), &["mutants-shard", "3/8", "out"]);
    assert_eq!(Text::lines(&refused.err), ["cannot run cargo: boom"]);
}

#[test]
fn changed_lines_are_written_and_tested_in_the_scope() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Script { changed: "+a line\n".to_owned(), ..Script::new() };
    let answer = scratch.answer(
        &fake,
        &[
            "mutants-changed",
            &scratch.file("tmp/c.diff"),
            &scratch.file("out"),
            "15/16",
            "--test-package=a",
            "--test-package=b",
        ],
    );
    assert_eq!(
        answer.lines(),
        Printed { exit: ExitCode::SUCCESS, out: vec!["mutants: none missed"], err: Vec::<&str>::new() }
    );
    assert_eq!(Text::lines(&scratch.read("tmp/c.diff")), ["+a line"]);
    assert_eq!(
        *fake.calls.borrow(),
        [
            Invocation::new("git", ["diff", "--no-renames", "HEAD^1", "HEAD", "--", "*.rs"]),
            Invocation::new(
                "cargo",
                [
                    "mutants",
                    "--in-diff",
                    scratch.file("tmp/c.diff").as_str(),
                    "--shard",
                    "15/16",
                    "--sharding",
                    "round-robin",
                    "--test-package=a",
                    "--test-package=b",
                    "--jobs",
                    "2",
                    "--colors",
                    "never",
                    "--output",
                    scratch.file("out").as_str(),
                ]
            ),
        ]
    );
    Ok(())
}

#[test]
fn missed_mutants_fail_the_step_and_are_listed() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.directory("out/mutants.out")?;
    scratch.write("out/mutants.out/missed.txt", "src/a.rs:1: replace f -> bool with true\n")?;
    let answer = scratch
        .answer(&Script::new(), &["mutants-changed", &scratch.file("c.diff"), &scratch.file("out"), "0/8", "-p"]);
    assert_eq!(answer.exit, ExitCode::from(2));
    assert_eq!(Text::lines(&answer.out), ["mutants missed:", "src/a.rs:1: replace f -> bool with true"]);
    Ok(())
}

#[test]
fn an_empty_missed_list_passes() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.directory("out/mutants.out")?;
    scratch.write("out/mutants.out/missed.txt", "")?;
    let answer = scratch
        .answer(&Script::new(), &["mutants-changed", &scratch.file("c.diff"), &scratch.file("out"), "0/8", "-p"]);
    assert_eq!((answer.exit, Text::lines(&answer.out)), (ExitCode::SUCCESS, vec!["mutants: none missed".to_owned()]));
    Ok(())
}

#[test]
fn a_change_that_reaches_no_crate_says_the_mutants_were_skipped() {
    let fake = Script::new();
    let answer = Scratch::fresh().answer(&fake, &["mutants-changed", "c.diff", "out", "0/8"]);
    assert_eq!(
        answer.lines(),
        Printed {
            exit: ExitCode::SUCCESS,
            out: vec!["mutants: skipped, no crate is reached by this change, so nothing was tested"],
            err: Vec::<&str>::new()
        }
    );
    assert!(fake.calls.borrow().is_empty());
}

#[test]
fn a_failed_start_while_testing_changed_lines_is_reported() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let arguments = ["mutants-changed", &scratch.file("c.diff"), &scratch.file("out"), "0/8", "-p"];
    assert_eq!(Text::lines(&scratch.answer(&Script::refusing("git"), &arguments).err), ["cannot run git: boom"]);
    assert_eq!(Text::lines(&scratch.answer(&Script::refusing("cargo"), &arguments).err), ["cannot run cargo: boom"]);
    scratch.write("blocker", "")?;
    let blocked =
        scratch.answer(&Script::new(), &["mutants-changed", &scratch.file("blocker/c.diff"), "out", "0/8", "-p"]);
    assert!(blocked.err.contains("blocker/c.diff"));
    Ok(())
}

#[test]
fn step_times_are_asked_of_github_for_the_job() {
    let fake = Script::answering(vec![Reply::to("gh api", "clippy: 12\ntest-run: 3\n")]);
    let answer = Scratch::fresh().answer(&fake, &["wall-times", "Studio-Terza/meta-dev", "77", "gates"]);
    assert_eq!(
        answer.lines(),
        Printed {
            exit: ExitCode::SUCCESS,
            out: vec!["wall time per step of gates, seconds:", "clippy: 12", "test-run: 3"],
            err: Vec::<&str>::new()
        }
    );
    assert_eq!(
        *fake.calls.borrow(),
        [Invocation::new(
            "gh",
            [
                "api",
                "repos/Studio-Terza/meta-dev/actions/runs/77/jobs",
                "--jq",
                ".jobs[] | select(.name == \"gates\") | .steps[] | select(.completed_at != null) | \"\\(.name): \\((.completed_at | fromdate) - (.started_at | fromdate))\"",
            ]
        )]
    );
    assert_eq!(
        Text::lines(&Scratch::fresh().answer(&Script::refusing("gh"), &["wall-times", "r", "1", "gates"]).err),
        ["cannot run gh: boom"]
    );
}

const LIMITS: &str =
    "[clippy]\nwarn = 11\nblock = 16\n[lint]\nwarn = 1.5\nblock = 3\n[test-run]\nwarn = 2\nblock = 3.5\n";
const CLIPPY_BODY: &str = "WARN: clippy took 12s, warn above 11s, block above 16s";
const TEST_RUN_BODY: &str = "BLOCK: test-run took 4s, warn above 2s, block above 3.5s";

struct Gh;

impl Gh {
    fn call(arguments: &[&str]) -> Invocation {
        Invocation::new("gh", arguments)
    }

    fn search(stage: &str) -> Invocation {
        Gh::call(&[
            "issue",
            "list",
            "--state",
            "open",
            "--limit",
            "100",
            "--search",
            &format!("\"{stage}\" in:title"),
            "--json",
            "number,title",
        ])
    }

    fn stage_issues(scratch: &Scratch, measured: &str) -> Result<Vec<String>, io::Error> {
        scratch.write("limits.toml", LIMITS)?;
        scratch.write("measured.toml", measured)?;
        Ok(["stage-issues", &scratch.file("limits.toml"), &scratch.file("measured.toml")].map(str::to_owned).to_vec())
    }
}

#[test]
fn a_stage_past_its_warn_line_opens_an_issue_titled_with_its_name() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let arguments = Gh::stage_issues(&scratch, "clippy = 12\nlint = 1\ntest-run = 4\n")?;
    let fake = Script::answering(vec![Reply::to("gh issue", "[]")]);
    let answer = scratch.answer(&fake, &arguments);
    assert_eq!(answer.exit, ExitCode::SUCCESS);
    assert_eq!(
        *fake.calls.borrow(),
        [
            Gh::search("clippy"),
            Gh::search("test-run"),
            Gh::call(&["issue", "create", "--title", "clippy", "--body", CLIPPY_BODY]),
            Gh::call(&["issue", "create", "--title", "test-run", "--body", TEST_RUN_BODY]),
        ]
    );
    Ok(())
}

#[test]
fn a_stage_that_already_has_an_open_issue_updates_that_issue_and_not_a_look_alike() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let arguments = Gh::stage_issues(&scratch, "clippy = 12\nlint = 1\ntest-run = 1\n")?;
    let listed = r#"[{"number":3,"title":"clippy fixes"},{"number":7,"title":"clippy"}]"#;
    let fake = Script::answering(vec![Reply::to("gh issue", listed)]);
    scratch.answer(&fake, &arguments);
    assert_eq!(fake.calls.borrow().last(), Some(&Gh::call(&["issue", "edit", "7", "--body", CLIPPY_BODY])));
    Ok(())
}

#[test]
fn stages_named_after_the_files_are_the_only_ones_judged() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let mut arguments = Gh::stage_issues(&scratch, "clippy = 12\n")?;
    arguments.push("clippy".to_owned());
    let fake = Script::answering(vec![Reply::to("gh issue", "[]")]);
    assert_eq!(scratch.answer(&fake, &arguments).exit, ExitCode::SUCCESS);
    assert_eq!(fake.calls.borrow().len(), 2);
    Ok(())
}

#[test]
fn stages_within_their_warn_line_open_nothing() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let arguments = Gh::stage_issues(&scratch, "clippy = 11\nlint = 1\ntest-run = 1\n")?;
    let fake = Script::new();
    assert_eq!(scratch.answer(&fake, &arguments).exit, ExitCode::SUCCESS);
    assert!(fake.calls.borrow().is_empty());
    Ok(())
}

#[test]
fn limits_that_cannot_be_judged_and_issues_that_cannot_be_handled_are_reported() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let arguments = Gh::stage_issues(&scratch, "clippy = 12\nlint = 1\ntest-run = 1\n")?;
    assert_eq!(Text::lines(&scratch.answer(&Script::refusing("gh"), &arguments).err), ["cannot run gh: boom"]);
    let garbled = Script::answering(vec![Reply::to("gh issue", "not json")]);
    assert!(scratch.answer(&garbled, &arguments).err.contains("cannot read the list of issues"));
    let failing = Script { exit: Exit::Failure(FAILED_WITH), ..Script::answering(vec![Reply::to("gh issue", "[]")]) };
    assert_eq!(scratch.answer(&failing, &arguments).exit, ExitCode::from(FAILED_WITH.get()));
    let unmeasured = Gh::stage_issues(&scratch, "clippy = 12\n")?;
    assert!(scratch.answer(&Script::new(), &unmeasured).err.contains("the limits cannot be judged"));
    Ok(())
}

const TREND_TITLE: &str = "compile-time trend: the cold build is slower than every run of the previous 14 days";
const TREND_BODY: [&str; 9] = [
    "The cold build, clippy and test compile from scratch, took 58.00s in all today; the 2 runs of the previous 14 days took 50.00s to 54.00s: slower than every one of them.",
    "",
    "Crates that grew most, seconds today against their average over those runs:",
    "+7.00 lint-core: 12.00 against 5.00",
    "+1.00 lint-new: 1.00 against 0.00",
    "",
    "Pull requests merged into main in the previous 14 days, as context:",
    "#254 strict boundary",
    "#250 bool strict",
];

struct Cold;

impl Cold {
    fn build(scratch: &Scratch, directory: &str, measured: &str, units: &[String]) -> Result<(), FileFailure> {
        let root = scratch.path().join(directory);
        scratch.placed(&root.join("measured.toml"), measured)?;
        ["clippy", "test-compile"].iter().try_for_each(|stage| {
            scratch.placed(
                &root.join(stage).join("cargo-timings").join("cargo-timing.html"),
                &Page::with(&units.join(",")),
            )
        })
    }

    fn fortnight(scratch: &Scratch, today: &str) -> io::Result<Vec<String>> {
        let grown = [
            Page::unit("lint-core", "6.0"),
            Page::unit("syn", "4.0"),
            Page::unit("ci", "1.0"),
            Page::unit("lint-new", "0.5"),
        ];
        Cold::build(scratch, "today", &format!("cold-clippy = 13\ncold-test-compile = {today}\n"), &grown)?;
        let first = [Page::unit("lint-core", "2.0"), Page::unit("syn", "4.0"), Page::unit("ci", "1.5")];
        Cold::build(scratch, "runs/11", "cold-clippy = 10\ncold-test-compile = 40\n", &first)?;
        let second = [Page::unit("lint-core", "3.0"), Page::unit("syn", "4.0")];
        Cold::build(scratch, "runs/12", "cold-clippy = 12\ncold-test-compile = 42\n", &second)?;
        Ok(["compile-trend", &scratch.file("today"), &scratch.file("runs")].map(str::to_owned).to_vec())
    }

    fn answers(runs: &str, issues: &str) -> Vec<Reply> {
        vec![
            Reply::to("gh run", runs),
            Reply::to("gh pr", "#254 strict boundary\n#250 bool strict\n"),
            Reply::to("gh issue", issues),
        ]
    }

    fn runs() -> Invocation {
        Gh::call(&[
            "run",
            "list",
            "--workflow",
            "cold-build.yml",
            "--branch",
            "main",
            "--status",
            "success",
            "--limit",
            "100",
            "--json",
            "databaseId,createdAt",
            "--jq",
            "[.[] | select((.createdAt | fromdate) > now - 14 * 86400) | .databaseId]",
        ])
    }

    fn download(scratch: &Scratch, run: &str) -> Invocation {
        Gh::call(&["run", "download", run, "--name", "cold-build", "--dir", &scratch.file(&format!("runs/{run}"))])
    }

    fn merged() -> Invocation {
        Gh::call(&[
            "pr",
            "list",
            "--state",
            "merged",
            "--base",
            "main",
            "--search",
            "sort:updated-desc",
            "--limit",
            "100",
            "--json",
            "number,title,mergedAt",
            "--jq",
            r##".[] | select((.mergedAt | fromdate) > now - 14 * 86400) | "#\(.number) \(.title)""##,
        ])
    }
}

#[test]
fn a_cold_build_slower_than_every_run_of_the_fortnight_opens_one_trend_issue_with_its_context()
-> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let arguments = Cold::fortnight(&scratch, "45")?;
    let fake = Script::answering(Cold::answers("[11, 12]", "[]"));
    let answer = scratch.answer(&fake, &arguments);
    let body = [TREND_BODY.join("\n"), String::from("\n")].concat();
    assert_eq!(answer.exit, ExitCode::SUCCESS, "{}", answer.err);
    assert_eq!(
        Text::lines(&answer.out),
        [TREND_BODY.to_vec(), vec![&format!("opening an issue: {TREND_TITLE}")]].concat()
    );
    assert_eq!(
        *fake.calls.borrow(),
        [
            Cold::runs(),
            Cold::download(&scratch, "11"),
            Cold::download(&scratch, "12"),
            Cold::merged(),
            Gh::search(TREND_TITLE),
            Gh::call(&["issue", "create", "--title", TREND_TITLE, "--body", &body]),
        ]
    );
    let listed = format!(r#"[{{"number":3,"title":"compile-time trend"}},{{"number":8,"title":"{TREND_TITLE}"}}]"#);
    let open = Script::answering(Cold::answers("[11, 12]", &listed));
    scratch.answer(&open, &arguments);
    assert_eq!(open.calls.borrow().last(), Some(&Gh::call(&["issue", "edit", "8", "--body", &body])));
    Ok(())
}

#[test]
fn a_cold_build_within_the_fortnight_or_without_one_opens_nothing() -> Result<(), Box<dyn Error>> {
    let cases = [
        ("39", "[11, 12]", "; the 2 runs of the previous 14 days took 50.00s to 54.00s, so no trend issue."),
        ("41", "[12, 11]", "; the 2 runs of the previous 14 days took 50.00s to 54.00s, so no trend issue."),
        ("45", "[]", "; no run of the previous 14 days to compare with, so no trend issue."),
    ];
    let concluded = cases
        .iter()
        .map(|(today, runs, said)| -> Result<(ExitCode, usize, bool), Box<dyn Error>> {
            let scratch = Scratch::new()?;
            let fake = Script::answering(Cold::answers(runs, "[]"));
            let answer = scratch.answer(&fake, &Cold::fortnight(&scratch, today)?);
            let only_runs =
                fake.calls.borrow().iter().all(|call| call.arguments().first().is_some_and(|first| first == "run"));
            Ok((answer.exit, answer.out.matches(&format!("{said}\n")).count(), only_runs))
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(concluded, cases.map(|_| (ExitCode::SUCCESS, 1, true)));
    Ok(())
}

#[test]
fn cold_builds_that_cannot_be_listed_downloaded_or_read_are_reported() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let arguments = Cold::fortnight(&scratch, "45")?;
    let undownloaded = Script::answering(Cold::answers("[13]", "[]"));
    assert!(scratch.answer(&undownloaded, &arguments).err.contains("cannot use"));
    let garbled = Script::answering(Cold::answers("nope", "[]"));
    assert!(scratch.answer(&garbled, &arguments).err.contains("cannot read the list of cold builds"));
    assert_eq!(Text::lines(&scratch.answer(&Script::refusing("gh"), &arguments).err), ["cannot run gh: boom"]);
    scratch.write("today/measured.toml", "cold-clippy = soon\n")?;
    assert!(scratch.answer(&Script::new(), &arguments).err.contains("cannot read the stage times in"));
    let absent = ["compile-trend", &scratch.file("none"), &scratch.file("runs")];
    assert!(scratch.answer(&Script::new(), &absent).err.contains("cannot use"));
    Ok(())
}

#[test]
fn a_cold_build_that_cannot_be_read_is_reported_by_the_path_of_its_file() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let arguments = Cold::fortnight(&scratch, "45")?;
    let undownloaded = Script::answering(Cold::answers("[13]", "[]"));
    assert!(scratch.answer(&undownloaded, &arguments).err.contains(&scratch.file("runs/13/measured.toml")));
    scratch.write("today/measured.toml", "cold-clippy = soon\n")?;
    assert!(scratch.answer(&Script::new(), &arguments).err.contains(&scratch.file("today/measured.toml")));
    Ok(())
}

#[test]
fn files_are_shown_under_their_names_and_absent_ones_are_said_to_be_absent() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("a.txt", "one\n")?;
    let answer = scratch.answer(&Script::new(), &["show", &scratch.file("a.txt"), &scratch.file("b.txt")]);
    let (shown, absent) = (format!("{}:", scratch.file("a.txt")), format!("{}:", scratch.file("b.txt")));
    let expected = vec![shown.as_str(), "one", absent.as_str(), "not produced"];
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: expected, err: Vec::<&str>::new() });
    Ok(())
}

#[derive(Debug)]
struct Case {
    kind: &'static str,
    changes: &'static str,
    labels: &'static str,
}

impl Case {
    fn of(changes: &'static str, labels: &'static str) -> Case {
        Case { kind: "change", changes, labels }
    }

    fn refactor(changes: &'static str, labels: &'static str) -> Case {
        Case { kind: "refactor", changes, labels }
    }
}

#[derive(Debug, PartialEq)]
struct Verdict {
    exit: ExitCode,
    out: String,
}

impl Verdict {
    fn passing() -> Verdict {
        Verdict { exit: ExitCode::SUCCESS, out: String::new() }
    }

    fn failing(out: &str) -> Verdict {
        Verdict { exit: ExitCode::FAILURE, out: out.to_owned() }
    }
}

struct Labelled;

impl Labelled {
    const HEADER: &'static str = "labels of pull request 121 against the merge base with main\n";
    const GOLDEN_ERROR: &'static str = "::error::this PR changes goldens of crates that nothing else it changes reaches, so it did not produce them; only the owner passes this, with the label golden-change\n";
    const SCENARIO_ERROR: &'static str = "::error::this PR adds files under tests/fixtures or tests/e2e of crates that nothing else it changes reaches; only the owner passes this, with the label new-scenario\n";
    const REFACTOR_ERROR: &'static str = "::error::this PR's title starts with refactor:, and a refactor leaves behaviour unchanged, so it changes no golden; only the owner passes this, with the label golden-change\n";

    fn verdict(case: &Case) -> Verdict {
        let fake = Script::answering(vec![
            Reply::to("git merge-base", "abc123\n"),
            Reply::to("git -c", case.changes),
            Reply::to("gh pr", &[case.kind, "\n", case.labels].concat()),
        ]);
        let scratch = Scratch::fresh();
        let answer = scratch.answer(&fake, &["labels", "main", "121", &scratch.file("metadata.json")]);
        Verdict { exit: answer.exit, out: answer.out.get(Self::HEADER.len()..).unwrap_or_default().to_owned() }
    }

    fn golden(path: &str) -> Verdict {
        Verdict::failing(&[path, "\n", Self::GOLDEN_ERROR].concat())
    }

    fn scenario(path: &str) -> Verdict {
        Verdict::failing(&[path, "\n", Self::SCENARIO_ERROR].concat())
    }

    fn refactor(path: &str) -> Verdict {
        Verdict::failing(&[path, "\n", Self::REFACTOR_ERROR].concat())
    }
}

laws::table!(
    goldens_and_new_scenarios_must_carry_their_labels,
    Labelled::verdict,
    [
        Case::of("", "") => Verdict::passing(),
        Case::of("M\tsrc/lib.rs\n", "") => Verdict::passing(),
        Case::of("M\tcrates/a/tests/goldens/x.txt\n", "") => Labelled::golden("crates/a/tests/goldens/x.txt"),
        Case::of("M\tcrates/a/tests/goldens/x.txt\n", "golden-change\n") => Verdict::passing(),
        Case::of("M\tcrates/a/tests/goldens/x.txt\n", "other\nnew-scenario\n") => Labelled::golden("crates/a/tests/goldens/x.txt"),
        Case::of("D\ttests/goldens/x.txt\n", "") => Labelled::golden("tests/goldens/x.txt"),
        Case::of("A\ttests/goldens/deep/er/x.txt\n", "") => Labelled::golden("tests/goldens/deep/er/x.txt"),
        Case::of("R090\ttests/goldens/x.txt\tdocs/x.txt\n", "") => Labelled::golden("docs/x.txt"),
        Case::of("R090\tdocs/x.txt\ttests/goldens/x.txt\n", "") => Labelled::golden("tests/goldens/x.txt"),
        Case::of("M\tgoldens/x.txt\n", "") => Verdict::passing(),
        Case::of("M\ttests/goldens\n", "") => Verdict::passing(),
        Case::of("M\ttests/other/goldens/x.txt\n", "") => Verdict::passing(),
        Case::of("M\tmytests/goldens/x.txt\n", "") => Verdict::passing(),
        Case::of("A\ttests/fixtures/a.txt\n", "") => Labelled::scenario("tests/fixtures/a.txt"),
        Case::of("A\ttests/e2e/a/b.txt\n", "") => Labelled::scenario("tests/e2e/a/b.txt"),
        Case::of("A\ttests/fixtures/a.txt\n", "new-scenario\n") => Verdict::passing(),
        Case::of("C100\ttests/fixtures/a.txt\ttests/fixtures/b.txt\n", "") => Labelled::scenario("tests/fixtures/b.txt"),
        Case::of("M\ttests/fixtures/a.txt\n", "") => Verdict::passing(),
        Case::of("D\ttests/fixtures/a.txt\n", "") => Verdict::passing(),
        Case::of("R100\ttests/fixtures/a.txt\ttests/e2e/a.txt\n", "") => Verdict::passing(),
        Case::of("R100\tsrc/a.txt\ttests/fixtures/a.txt\n", "") => Labelled::scenario("tests/fixtures/a.txt"),
        Case::of("R100\ttests/fixtures/a.txt\tsrc/a.txt\n", "") => Verdict::passing(),
        Case::of("A\ttests/fixture/a.txt\n", "") => Verdict::passing(),
        Case::of("A\tfixtures/a.txt\n", "") => Verdict::passing(),
        Case::of("M\ttests/goldens/x.txt\nA\ttests/fixtures/y.txt\n", "") => Verdict::failing(
            &["tests/goldens/x.txt\n", Labelled::GOLDEN_ERROR, "tests/fixtures/y.txt\n", Labelled::SCENARIO_ERROR].concat(),
        ),
        Case::of("M\ttests/goldens/x.txt\nA\ttests/fixtures/y.txt\n", "golden-change\n") => Labelled::scenario("tests/fixtures/y.txt"),
        Case::of("garbage\nX\n\n", "") => Verdict::passing(),
    ]
);

laws::table!(
    goldens_and_new_scenarios_pass_inside_the_reach_of_the_rest_of_the_change,
    Labelled::verdict,
    [
        Case::of("M\tcrates/leaf/src/lib.rs\nM\tcrates/leaf/tests/goldens/x.txt\n", "") => Verdict::passing(),
        Case::of("M\tcrates/core/src/lib.rs\nM\tcrates/tested-through/tests/goldens/x.txt\n", "") => Verdict::passing(),
        Case::of("M\tcrates/leaf/tests/behaviour.rs\nD\tcrates/leaf/tests/goldens/x.txt\n", "") => Verdict::passing(),
        Case::of("M\tCargo.lock\nM\ttests/goldens/x.txt\n", "") => Verdict::passing(),
        Case::of("M\tcrates/middle/src/lib.rs\nM\tcrates/core/tests/goldens/x.txt\n", "") => Labelled::golden("crates/core/tests/goldens/x.txt"),
        Case::of("M\tcrates/leaf/tests/goldens/x.txt\nM\tREADME.md\n", "") => Labelled::golden("crates/leaf/tests/goldens/x.txt"),
        Case::of("M\tcrates/leaf/tests/goldens/x.txt\nM\tcrates/core/tests/goldens/x.txt\n", "") => Verdict::failing(
            &["crates/leaf/tests/goldens/x.txt\n", "crates/core/tests/goldens/x.txt\n", Labelled::GOLDEN_ERROR].concat(),
        ),
        Case::of("M\tcrates/middle/src/lib.rs\nM\tcrates/core/tests/goldens/x.txt\n", "golden-change\n") => Verdict::passing(),
        Case::of("M\tcrates/leaf/src/lib.rs\nR100\tcrates/leaf/tests/goldens/x.txt\tcrates/core/tests/goldens/x.txt\n", "") => Labelled::golden("crates/core/tests/goldens/x.txt"),
        Case::of("M\tcrates/leaf/src/lib.rs\nA\tcrates/leaf/tests/fixtures/a.txt\n", "") => Verdict::passing(),
        Case::of("M\tcrates/core/src/lib.rs\nA\tcrates/middle/tests/e2e/a.txt\n", "") => Verdict::passing(),
        Case::of("M\tcrates/leaf/src/lib.rs\nA\tcrates/core/tests/fixtures/a.txt\n", "") => Labelled::scenario("crates/core/tests/fixtures/a.txt"),
        Case::of("M\tcrates/leaf/src/lib.rs\nA\tcrates/core/tests/fixtures/a.txt\n", "new-scenario\n") => Verdict::passing(),
        Case::of("M\tcrates/leaf/tests/fixtures/a.txt\nM\tcrates/leaf/tests/goldens/x.txt\n", "") => Verdict::passing(),
    ]
);

laws::table!(
    a_pull_request_titled_refactor_changes_no_golden,
    Labelled::verdict,
    [
        Case::refactor("M\tcrates/leaf/src/lib.rs\nM\tcrates/leaf/tests/goldens/x.txt\n", "") => Labelled::refactor("crates/leaf/tests/goldens/x.txt"),
        Case::refactor("M\tcrates/leaf/src/lib.rs\nM\tcrates/leaf/tests/goldens/x.txt\n", "golden-change\n") => Verdict::passing(),
        Case::refactor("M\tcrates/leaf/src/lib.rs\n", "") => Verdict::passing(),
        Case::refactor("M\tcrates/leaf/src/lib.rs\nA\tcrates/leaf/tests/fixtures/a.txt\n", "") => Verdict::passing(),
        Case::of("M\tcrates/leaf/src/lib.rs\nM\tcrates/leaf/tests/goldens/x.txt\n", "refactor\n") => Verdict::passing(),
    ]
);

#[test]
fn labels_are_checked_against_the_merge_base_with_the_base_branch() {
    let fake = Script::answering(vec![Reply::to("git merge-base", "abc123\n")]);
    let scratch = Scratch::fresh();
    let metadata = scratch.file("metadata.json");
    let answer = scratch.answer(&fake, &["labels", "release", "7", &metadata]);
    assert_eq!(Text::lines(&answer.out), ["labels of pull request 7 against the merge base with release"]);
    assert_eq!(scratch.read("metadata.json"), METADATA);
    assert_eq!(
        *fake.calls.borrow(),
        [
            Invocation::new("git", ["merge-base", "origin/release", "HEAD"]),
            Invocation::new(
                "git",
                [
                    "-c",
                    "core.quotePath=false",
                    "diff",
                    "--name-status",
                    "--find-renames",
                    "--find-copies",
                    "abc123",
                    "HEAD"
                ],
            ),
            Invocation::new(
                "gh",
                [
                    "pr",
                    "view",
                    "7",
                    "--json",
                    "title,labels",
                    "--jq",
                    r#"if (.title | startswith("refactor:")) then "refactor" else "change" end, .labels[].name"#
                ]
            ),
            Invocation::new("cargo", ["metadata", "--locked", "--no-deps", "--format-version", "1"]),
        ]
    );
    assert_eq!(
        Text::lines(&scratch.answer(&Script::refusing("git"), &["labels", "main", "1", &metadata]).err),
        ["cannot run git: boom"]
    );
    assert_eq!(
        Text::lines(&scratch.answer(&Script::refusing("gh"), &["labels", "main", "1", &metadata]).err),
        ["cannot run gh: boom"]
    );
    assert_eq!(
        Text::lines(&scratch.answer(&Script::refusing("cargo"), &["labels", "main", "1", &metadata]).err),
        ["cannot run cargo: boom"]
    );
}

struct Body;

impl Body {
    const FORMS: &'static str = "write a line spec: none, spec: owner <uuid prefix of 8 or more hex digits>, or spec: <project>#D<n> or <project>#L<n> (several comma-separated); a PR that repairs a red main writes the line repair: main instead\n";
    const REPAIRING: &'static str =
        "repairing main: the body has the line repair: main, so the spec line is not checked\n";

    fn checked(body: &&str) -> Verdict {
        let fake = Script::answering(vec![Reply::to("gh pr", body)]);
        let answer = Scratch::fresh().answer(&fake, &["spec-line", "172"]);
        Verdict { exit: answer.exit, out: answer.out }
    }

    fn formed(line: &str) -> Verdict {
        Verdict { exit: ExitCode::SUCCESS, out: ["spec line well formed: ", line, "\n"].concat() }
    }

    fn repairing() -> Verdict {
        Verdict { exit: ExitCode::SUCCESS, out: Self::REPAIRING.to_owned() }
    }

    fn malformed(lines: &[&str]) -> Verdict {
        let listed: String = lines.iter().map(|line| ["spec line malformed: ", line, "\n"].concat()).collect();
        Verdict::failing(&[listed.as_str(), "::error::no spec line of the body is well formed; ", Self::FORMS].concat())
    }

    fn missing() -> Verdict {
        Verdict::failing(
            &["spec line missing\n::error::the body has no line starting \"spec: \"; ", Self::FORMS].concat(),
        )
    }
}

laws::table!(
    a_pull_request_body_carries_a_well_formed_spec_line_unless_it_repairs_main,
    Body::checked,
    [
        "spec: none\n" => Body::formed("spec: none"),
        "spec: local-tools#D23\n\nMore text.\n" => Body::formed("spec: local-tools#D23"),
        "spec: lint-port#L5\n" => Body::formed("spec: lint-port#L5"),
        "spec: test-tooling#D6, test-tooling#D7\n" => Body::formed("spec: test-tooling#D6, test-tooling#D7"),
        "spec: a#D1,b2#L10\n" => Body::formed("spec: a#D1,b2#L10"),
        "spec: owner 96d13317\n" => Body::formed("spec: owner 96d13317"),
        "spec: owner 4a82c67a-1b2c\n" => Body::formed("spec: owner 4a82c67a-1b2c"),
        "spec: none\r\nbody\r\n" => Body::formed("spec: none"),
        "Intro.\nspec: rust-only#D2  \n" => Body::formed("spec: rust-only#D2"),
        "spec: wrong\nspec: none\n" => Body::formed("spec: none"),
        "repair: main\n" => Body::repairing(),
        "spec: wrong\nrepair: main\n" => Body::repairing(),
        "spec: none\nrepair: main\n" => Body::repairing(),
        "" => Body::missing(),
        "Implements D1 of the spec.\n" => Body::missing(),
        "Spec: none\n" => Body::missing(),
        " spec: none\n" => Body::missing(),
        "spec:none\n" => Body::missing(),
        "repair: main later\n" => Body::missing(),
        "Repair: main\n" => Body::missing(),
        "spec: \n" => Body::missing(),
        "spec: None\n" => Body::malformed(&["spec: None"]),
        "spec: none, a#D1\n" => Body::malformed(&["spec: none, a#D1"]),
        "spec: owner 96d1331\n" => Body::malformed(&["spec: owner 96d1331"]),
        "spec: owner 96d1331g\n" => Body::malformed(&["spec: owner 96d1331g"]),
        "spec: owner 96d13317x\n" => Body::malformed(&["spec: owner 96d13317x"]),
        "spec: owner 96d1-3317\n" => Body::malformed(&["spec: owner 96d1-3317"]),
        "spec: owner\n" => Body::malformed(&["spec: owner"]),
        "spec: owner \n" => Body::malformed(&["spec: owner"]),
        "spec: spec: \n" => Body::malformed(&["spec: spec:"]),
        "spec: Lint-port#D3\n" => Body::malformed(&["spec: Lint-port#D3"]),
        "spec: lint_port#D3\n" => Body::malformed(&["spec: lint_port#D3"]),
        "spec: #D3\n" => Body::malformed(&["spec: #D3"]),
        "spec: lint-port#D\n" => Body::malformed(&["spec: lint-port#D"]),
        "spec: lint-port#X3\n" => Body::malformed(&["spec: lint-port#X3"]),
        "spec: lint-port#d3\n" => Body::malformed(&["spec: lint-port#d3"]),
        "spec: lint-port#D3a\n" => Body::malformed(&["spec: lint-port#D3a"]),
        "spec: lint-port D3\n" => Body::malformed(&["spec: lint-port D3"]),
        "spec: a#D1,\n" => Body::malformed(&["spec: a#D1,"]),
        "spec: a#D1 b#D2\n" => Body::malformed(&["spec: a#D1 b#D2"]),
        "spec: a#D1 and more\nspec: x\n" => Body::malformed(&["spec: a#D1 and more", "spec: x"]),
    ]
);

#[test]
fn the_spec_line_is_read_from_the_body_of_the_pull_request() {
    let fake = Script::answering(vec![Reply::to("gh pr", "spec: none\n")]);
    let answer = Scratch::fresh().answer(&fake, &["spec-line", "9"]);
    assert_eq!(answer.exit, ExitCode::SUCCESS);
    assert_eq!(*fake.calls.borrow(), [Invocation::new("gh", ["pr", "view", "9", "--json", "body", "--jq", ".body"])]);
    let refused = Scratch::fresh().answer(&Script::refusing("gh"), &["spec-line", "9"]);
    assert_eq!(Text::lines(&refused.err), ["cannot run gh: boom"]);
    assert_eq!(refused.err.chars().last(), Some('\n'));
}

#[test]
fn shard_results_are_counted_and_the_missed_and_timed_out_are_listed() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.directory("shards/shard-0")?;
    scratch.directory("shards/shard-1")?;
    scratch.write("shards/shard-0/caught.txt", "c1\nc2\n")?;
    scratch.write("shards/shard-0/missed.txt", "m2\n")?;
    scratch.write("shards/shard-1/caught.txt", "c3\n")?;
    scratch.write("shards/shard-1/missed.txt", "m1\n")?;
    scratch.write("shards/shard-1/timeout.txt", "t1\n")?;
    scratch.write("sum", "before\n")?;
    let answer = scratch.answer(&Script::new(), &["shard-summary", &scratch.file("shards"), &scratch.file("sum")]);
    let expected = ["caught: 3", "missed: 2", "timeout: 1", "unviable: 0", "```", "m1", "m2", "t1", "```"];
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: expected.to_vec(), err: Vec::<&str>::new() });
    assert_eq!(Text::lines(&scratch.read("sum")), [["before"].as_slice(), expected.as_slice()].concat());
    Ok(())
}

#[test]
fn a_shard_summary_without_shards_is_reported() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let answer = scratch.answer(&Script::new(), &["shard-summary", &scratch.file("absent"), &scratch.file("sum")]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("absent"));
    assert_eq!(scratch.kind_of("sum"), Kind::Missing);
    Ok(())
}

#[test]
fn the_matrix_is_written_shown_and_added_to_the_summary() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.directory("shards/shard-0")?;
    scratch.directory("sources/crates")?;
    scratch.write(
        "shards/shard-0/matrix.json",
        r#"{"baseline":["t :: one"],"mutants":[{"name":"m1","summary":"MissedMutant","failed":[]}]}"#,
    )?;
    let arguments = [
        "matrix",
        &scratch.file("shards"),
        &scratch.file("sources"),
        &scratch.file("out/matrix"),
        &scratch.file("sum"),
    ];
    let answer = scratch.answer(&Script::new(), &arguments);
    assert_eq!((answer.exit, answer.err.as_str()), (ExitCode::SUCCESS, ""));
    assert!(answer.out.contains("1 mutants, 1 tests, 0 golden tests."));
    assert_eq!(scratch.read("out/matrix/report.md"), answer.out);
    assert_eq!(scratch.read("sum"), answer.out);
    assert!(scratch.read("out/matrix/report.json").contains("\"surviving\""));
    Ok(())
}

#[test]
fn a_matrix_that_cannot_be_made_is_reported_with_its_reason() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let arguments =
        ["matrix", &scratch.file("shards"), &scratch.file("sources"), &scratch.file("out"), &scratch.file("sum")];
    let answer = scratch.answer(&Script::new(), &arguments);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("the mutation matrix failed: "));
    scratch.write("blocker", "")?;
    let blocked = scratch.answer(&Script::new(), &["matrix", "s", "t", &scratch.file("blocker/out"), "sum"]);
    assert!(blocked.err.contains("blocker/out"));
    Ok(())
}

#[test]
fn a_tee_writes_to_both_and_stops_at_the_first_failure() -> Result<(), Box<dyn Error>> {
    let (mut first, mut second) = (Vec::new(), Vec::new());
    {
        let mut tee = Tee::new(&mut first, &mut second);
        assert_eq!(tee.write(b"abc")?, 3);
        tee.flush()?;
    }
    assert_eq!((first.as_slice(), second.as_slice()), (b"abc".as_slice(), b"abc".as_slice()));
    let mut full: [u8; 2] = [0; 2];
    let mut limited = Tee::new(Vec::new(), full.as_mut_slice());
    assert!(limited.write(b"abc").is_err());
    assert!(Tee::new(Vec::new(), Unflushable).flush().is_err());
    assert!(Tee::new(Unflushable, Vec::new()).flush().is_err());
    Ok(())
}

struct Unflushable;

impl Write for Unflushable {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("stuck"))
    }
}

#[derive(Debug)]
struct Start(Exit);

struct Followed;

impl Followed {
    fn run(start: &Start) -> Result<Exit, ()> {
        start.0.followed_by(|| Ok(Exit::Failure(FAILED_WITH)))
    }

    fn excusing(code: &Code) -> Exit {
        Exit::Failure(NonZeroU8::new(code.0).unwrap_or(FAILED_WITH)).excusing(&[FAILED_WITH, NonZeroU8::MIN])
    }
}

laws::table!(
    a_step_follows_only_a_success,
    Followed::run,
    [
        Start(Exit::Success) => Ok(Exit::Failure(FAILED_WITH)),
        Start(Exit::Failure(NonZeroU8::MIN)) => Ok(Exit::Failure(NonZeroU8::MIN)),
    ]
);

laws::table!(
    only_the_excused_failures_become_successes,
    Followed::excusing,
    [
        Code(1) => Exit::Success,
        Code(3) => Exit::Success,
        Code(2) => Exit::Failure(NonZeroU8::MIN.saturating_add(1)),
        Code(4) => Exit::Failure(NonZeroU8::MIN.saturating_add(3)),
    ]
);

#[derive(Debug, PartialEq)]
struct Printed<Line> {
    exit: ExitCode,
    out: Vec<Line>,
    err: Vec<Line>,
}

impl Printed<String> {
    fn shown(&self) -> Printed<&str> {
        Printed {
            exit: self.exit,
            out: self.out.iter().map(String::as_str).collect(),
            err: self.err.iter().map(String::as_str).collect(),
        }
    }
}

impl PartialEq<Printed<&str>> for Printed<String> {
    fn eq(&self, wanted: &Printed<&str>) -> bool {
        self.shown() == *wanted
    }
}

struct Text;

impl Text {
    fn owned(text: &str) -> Vec<String> {
        text.parse::<text_lines::Lines>().into_iter().flatten().collect()
    }

    fn lines(text: &impl AsRef<str>) -> Vec<String> {
        text.as_ref().parse::<text_lines::Lines>().into_iter().flatten().collect()
    }
}

struct Scratch {
    files: Memory,
    root: PathBuf,
}

impl Scratch {
    fn new() -> Result<Scratch, FileFailure> {
        let scratch = Scratch { files: Memory::default(), root: PathBuf::from("/scratch") };
        scratch.files.create_directories(&scratch.root)?;
        Ok(scratch)
    }

    fn fresh() -> Scratch {
        Scratch::new().unwrap_or_else(|_| unreachable!("an empty memory takes a directory"))
    }

    fn answer<Argument: AsRef<str>>(&self, script: &Script, arguments: &[Argument]) -> Answer {
        Answer::to(&Fake { script, scratch: self }, arguments)
    }

    fn placed(&self, path: &Path, text: &str) -> Result<(), FileFailure> {
        path.parent().map_or(Ok(()), |parent| self.files.create_directories(parent))?;
        self.files.write(path, Bytes::from(text))
    }

    fn directory(&self, name: &str) -> Result<(), FileFailure> {
        self.files.create_directories(&self.root.join(name))
    }

    fn file(&self, name: &str) -> String {
        self.root.join(name).display().to_string()
    }

    fn read(&self, name: &str) -> String {
        self.files.read_text(&self.root.join(name)).map_or_default(|text| text.to_string())
    }

    fn write(&self, name: &str, text: &str) -> Result<(), FileFailure> {
        self.files.write(&self.root.join(name), Bytes::from(text))
    }

    fn kind_of(&self, name: &str) -> Kind {
        self.files.kind(&self.root.join(name))
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

struct Sqlite;

struct Built {
    answer: Answer,
    calls: Vec<Vec<String>>,
    library: String,
}

impl Sqlite {
    const LISTED: &str = r#"{"packages": [
        {"name": "rusqlite", "manifest_path": "/r/rusqlite-0.37.0/Cargo.toml"},
        {"name": "libsqlite3-sys", "manifest_path": "/r/libsqlite3-sys-0.37.0/Cargo.toml"}
    ]}"#;
    const UNLISTED: &str = r#"{"packages": [{"name": "rusqlite", "manifest_path": "/r/rusqlite-0.37.0/Cargo.toml"}]}"#;
    const METADATA: [&str; 5] = ["cargo", "metadata", "--locked", "--format-version", "1"];
    const COMPILED: [&str; 24] = [
        "cc",
        "-O2",
        "-DSQLITE_CORE",
        "-DSQLITE_DEFAULT_FOREIGN_KEYS=1",
        "-DSQLITE_ENABLE_API_ARMOR",
        "-DSQLITE_ENABLE_COLUMN_METADATA",
        "-DSQLITE_ENABLE_DBSTAT_VTAB",
        "-DSQLITE_ENABLE_FTS3",
        "-DSQLITE_ENABLE_FTS3_PARENTHESIS",
        "-DSQLITE_ENABLE_FTS5",
        "-DSQLITE_ENABLE_JSON1",
        "-DSQLITE_ENABLE_LOAD_EXTENSION=1",
        "-DSQLITE_ENABLE_MEMORY_MANAGEMENT",
        "-DSQLITE_ENABLE_RTREE",
        "-DSQLITE_ENABLE_STAT4",
        "-DSQLITE_SOUNDEX",
        "-DSQLITE_THREADSAFE=1",
        "-DSQLITE_USE_URI",
        "-DHAVE_USLEEP=1",
        "-DHAVE_ISNAN",
        "-D_POSIX_THREAD_SAFE_FUNCTIONS",
        "-DSQLITE_DEFAULT_MEMSTATUS=0",
        "-DHAVE_LOCALTIME_R",
        "-c",
    ];

    fn built(metadata: &str, exit: u8) -> Result<Built, Box<dyn Error>> {
        let scratch = Scratch::new()?;
        let script = Script { metadata: metadata.to_owned(), ..Script::exiting(exit) };
        let library = scratch.file("sqlite");
        let answer = scratch.answer(&script, &["sqlite", &library]);
        let calls = script.calls.borrow().iter().map(Sqlite::words).collect();
        Ok(Built { answer, calls, library })
    }

    fn words(call: &Invocation) -> Vec<String> {
        iter::once(call.program())
            .chain(call.arguments().iter().map(OsString::as_os_str))
            .map(|word| word.to_string_lossy().into_owned())
            .collect()
    }

    fn owned(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| (*word).to_owned()).collect()
    }
}

#[test]
fn sqlite_is_compiled_from_the_source_cargo_metadata_names_and_archived_into_the_library_directory()
-> Result<(), Box<dyn Error>> {
    let Built { answer, calls, library } = Sqlite::built(Sqlite::LISTED, 0)?;
    let object = format!("{library}/sqlite3.o");
    let compiled = Sqlite::owned(&Sqlite::COMPILED)
        .into_iter()
        .chain(["/r/libsqlite3-sys-0.37.0/sqlite3/sqlite3.c".to_owned(), "-o".to_owned(), object.clone()])
        .collect();
    let archived = vec!["ar".to_owned(), "rcs".to_owned(), format!("{library}/libsqlite3.a"), object];
    assert_eq!(answer.exit, ExitCode::SUCCESS);
    assert_eq!(calls, [Sqlite::owned(&Sqlite::METADATA), compiled, archived]);
    Ok(())
}

#[test]
fn sqlite_without_libsqlite3_sys_in_the_metadata_is_refused_and_compiles_nothing() -> Result<(), Box<dyn Error>> {
    let Built { answer, calls, .. } = Sqlite::built(Sqlite::UNLISTED, 0)?;
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("no libsqlite3-sys package"));
    assert_eq!(calls, [Sqlite::owned(&Sqlite::METADATA)]);
    Ok(())
}

#[test]
fn sqlite_whose_compile_fails_is_not_archived() -> Result<(), Box<dyn Error>> {
    let Built { answer, calls, .. } = Sqlite::built(Sqlite::LISTED, 1)?;
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert_eq!(
        calls.iter().map(|call| call.first().cloned().unwrap_or_default()).collect::<Vec<String>>(),
        ["cargo", "cc"]
    );
    Ok(())
}

#[test]
fn a_build_script_waits_for_the_host_build_of_its_build_dependencies() -> Result<(), Box<dyn Error>> {
    let page = Page::with(
        &[
            Page::of("heavy", "1.0.0", "", "0.5"),
            Page::of("sys", "0.3.0", " build-script", "0.1"),
            Page::of("sys", "0.3.0", " build-script (run)", "0.2"),
            Page::of("sys", "0.3.0", " (check)", "1.0"),
        ]
        .join(","),
    );
    let Graph { answer, .. } = Graph::measured(&page, TIMINGS_METADATA)?;
    let expected = [
        "graph-path 0.30 seconds, the longest chain of clippy units, counting only units sccache never caches:",
        "0.00 heavy (build)",
        "0.10 sys build-script",
        "0.20 sys build-script (run)",
        "graph-work 0.30 seconds, the sum of the same units",
    ];
    assert_eq!(Text::lines(&answer.out), expected);
    Ok(())
}

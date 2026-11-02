use std::cell::RefCell;
use std::error::Error;
use std::ffi::OsString;
use std::io::{self, Write};
use std::num::NonZeroU8;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use ci::Cli;
use ci::ports::{Exit, Invocation, Ports, Run, Tee};
use ports::Scratch;

const METADATA: &str = include_str!("../../reach/tests/metadata/workspace.json");
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
const HISTORY_LINE: &str = "abc123\treach 1 of 5\tfan-in core=2 middle=1 leaf=0 serde-user=0 tested-through=0";
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

struct Fake {
    exit: Exit,
    elapsed: Duration,
    changed: String,
    metadata: String,
    answers: Vec<Reply>,
    failing: Option<&'static str>,
    logged_text: String,
    calls: RefCell<Vec<Invocation>>,
    logs: RefCell<Vec<PathBuf>>,
}

impl Fake {
    fn new() -> Fake {
        Fake {
            exit: Exit::Success,
            elapsed: Duration::from_millis(1_500),
            changed: String::new(),
            metadata: METADATA.to_owned(),
            answers: Vec::new(),
            failing: None,
            logged_text: String::new(),
            calls: RefCell::new(Vec::new()),
            logs: RefCell::new(Vec::new()),
        }
    }

    fn refusing(program: &'static str) -> Fake {
        Fake { failing: Some(program), ..Fake::new() }
    }

    fn answering(answers: Vec<Reply>) -> Fake {
        Fake { answers, ..Fake::new() }
    }

    fn exiting(code: u8) -> Fake {
        Fake { exit: NonZeroU8::new(code).map_or(Exit::Success, Exit::Failure), ..Fake::new() }
    }

    fn recorded(&self, invocation: &Invocation) -> io::Result<()> {
        self.calls.borrow_mut().push(invocation.clone());
        match self.failing {
            Some(program) if invocation.program() == program => Err(io::Error::other("boom")),
            _ => Ok(()),
        }
    }

    fn answer_to(&self, invocation: &Invocation) -> String {
        let key = [
            invocation.program().to_string_lossy().as_ref(),
            " ",
            invocation.arguments().first().map(|first| first.to_string_lossy()).unwrap_or_default().as_ref(),
        ]
        .concat();
        self.answers.iter().find(|reply| reply.key == key).map_or_else(
            || if invocation.program() == "git" { self.changed.clone() } else { self.metadata.clone() },
            |reply| reply.text.clone(),
        )
    }
}

impl Ports for Fake {
    fn run(&self, invocation: &Invocation) -> io::Result<Run> {
        self.recorded(invocation)?;
        Ok(Run::new(self.exit, self.elapsed))
    }

    fn output(&self, invocation: &Invocation) -> io::Result<String> {
        self.recorded(invocation)?;
        Ok(self.answer_to(invocation))
    }

    fn logged(&self, invocation: &Invocation, log: &Path) -> io::Result<Run> {
        self.recorded(invocation)?;
        self.logs.borrow_mut().push(log.to_path_buf());
        Scratch::placed(log, &self.logged_text)?;
        Ok(Run::new(self.exit, self.elapsed))
    }
}

#[derive(Debug, PartialEq)]
struct Answer {
    exit: ExitCode,
    out: String,
    err: String,
}

impl Answer {
    fn lines(&self) -> Printed<'_> {
        Printed { exit: self.exit, out: Text::lines(&self.out), err: Text::lines(&self.err) }
    }

    fn to<Argument: AsRef<str>>(fake: &Fake, arguments: &[Argument]) -> Answer {
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
        let answer = Answer::to(&Fake::new(), arguments);
        answer.exit == ExitCode::FAILURE && answer.out.is_empty() && answer.err.contains("usage: ci ")
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

struct Sample;

impl Sample {
    fn timed_line(elapsed: &Duration) -> Vec<String> {
        let scratch = Scratch::fresh();
        let fake = Fake { elapsed: *elapsed, ..Fake::new() };
        Answer::to(&fake, &["timed", &scratch.file("m.toml"), "clippy", "cargo"]);
        Text::owned(&scratch.read("m.toml"))
    }

    fn libtest_line(output: &&str) -> Vec<String> {
        let scratch = Scratch::fresh();
        scratch.write("out.txt", output).ok();
        Answer::to(&Fake::new(), &["libtest-seconds", &scratch.file("m.toml"), "test-run", &scratch.file("out.txt")]);
        Text::owned(&scratch.read("m.toml"))
    }

    fn reach_report(changed: &&str) -> Vec<String> {
        let scratch = Scratch::fresh();
        let fake = Fake { changed: (*changed).to_owned(), ..Fake::new() };
        Text::owned(&Answer::to(&fake, &Reach::into(&scratch, "pull_request", "feature")).out)
    }

    fn reach_into_main(changed: &&str) -> Vec<String> {
        let scratch = Scratch::fresh();
        let fake = Fake { changed: (*changed).to_owned(), ..Fake::new() };
        Text::owned(&Answer::to(&fake, &Reach::into(&scratch, "pull_request", "main")).out)
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
        "test result: ok. 3 passed; 0 failed; finished in 0.50s\n" => vec!["test-run = 0.50"],
        "test result: ok. 1 passed; finished in 0.50s\ntest result: ok. 2 passed; finished in 1.25s\n" => vec!["test-run = 1.75"],
        "running 3 tests\n...\ntest result: ok. finished in 2.00s\nfinished in 9.00s\n" => vec!["test-run = 2.00"],
        "test result: ok. 1 passed; 0 failed\n" => vec!["test-run = 0.00"],
        "" => Vec::<&str>::new(),
        "no result lines here\n" => Vec::<&str>::new(),
        "test result: ok. finished in abc\n" => vec!["test-run = 0.00"],
        "test result: ok. finished in 3s\n" => vec!["test-run = 3.00"],
        "test result: ok. finished in 0.5\n" => vec!["test-run = 0.50"],
        "test result: ok. finished in 1s in 2s\n" => vec!["test-run = 3.00"],
        "  test result: ok. finished in 9s\n" => Vec::<&str>::new(),
        "test results: ok. finished in 4s\n" => vec!["test-run = 4.00"],
        "test result: ok. finished in\n" => vec!["test-run = 0.00"],
        "test result: ok. finishedin 5s\n" => vec!["test-run = 0.00"],
        "test result: ok. in 1.5.5s\n" => vec!["test-run = 0.00"],
        "test resul: ok. finished in 5s\n" => Vec::<&str>::new(),
        "test result: ok. finished  in\t7s\n" => vec!["test-run = 7.00"],
        "test result: ok. finished in 0.004s\n" => vec!["test-run = 0.00"],
    ]
);

laws::table!(
    the_scope_is_what_the_changed_files_reach,
    Sample::reach_report,
    [
        "crates/leaf/src/lib.rs\n" => LEAF_REPORT.to_vec(),
        "crates/middle/src/lib.rs\n" => TWO_REPORT.to_vec(),
        "crates/middle/src/lib.rs\ncrates/middle/src/other.rs\n" => TWO_REPORT.to_vec(),
        "README.md\n" => EMPTY_REPORT_AND_NOTES.to_vec(),
        "" => EMPTY_REPORT_AND_NOTES.to_vec(),
    ]
);

laws::table!(
    a_pull_request_into_main_runs_the_whole_workspace_but_reports_the_reach_for_mutants,
    Sample::reach_into_main,
    [
        "crates/leaf/src/lib.rs\n" => vec!["REACH=1 of 5 crates", "BUILD_SCOPE=--workspace", "MUTANTS_SCOPE=--test-package=leaf"],
        "README.md\n" => vec!["REACH=0 of 5 crates", "BUILD_SCOPE=--workspace", "MUTANTS_SCOPE=", "skipped: MUTANTS_SCOPE is empty, so the steps that use it do not run"],
    ]
);

#[test]
fn a_timed_stage_runs_the_program_with_its_arguments() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Fake::new();
    let answer = Answer::to(&fake, &["timed", &scratch.file("m.toml"), "clippy", "cargo", "clippy", "--all-targets"]);
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
    Answer::to(&Fake::new(), &["timed", &scratch.file("m.toml"), "clippy", "cargo"]);
    Answer::to(&Fake::new(), &["timed", &scratch.file("m.toml"), "lint", "cargo"]);
    assert_eq!(Text::lines(&scratch.read("m.toml")), ["fmt = 0.10", "clippy = 1.50", "lint = 1.50"]);
    Ok(())
}

#[test]
fn a_failing_program_fails_the_step_with_its_code_and_is_still_measured() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Fake { exit: Exit::Failure(FAILED_WITH), ..Fake::new() };
    let answer = Answer::to(&fake, &["timed", &scratch.file("m.toml"), "clippy", "cargo"]);
    assert_eq!(answer.exit, ExitCode::from(FAILED_WITH.get()));
    assert_eq!(Text::lines(&scratch.read("m.toml")), ["clippy = 1.50"]);
    Ok(())
}

#[test]
fn a_program_that_cannot_start_is_reported_and_not_measured() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let answer = Answer::to(&Fake::refusing("cargo"), &["timed", &scratch.file("m.toml"), "clippy", "cargo"]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert_eq!(Text::lines(&answer.err), ["cannot run cargo: boom"]);
    assert_eq!(scratch.read("m.toml"), "");
    Ok(())
}

#[test]
fn a_measured_file_that_cannot_be_written_is_reported() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("blocker", "")?;
    let answer = Answer::to(&Fake::new(), &["timed", &scratch.file("blocker/m.toml"), "clippy", "cargo"]);
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
    let answer =
        Answer::to(&Fake::new(), &["libtest-seconds", &scratch.file("m.toml"), "test-run", &scratch.file("out.txt")]);
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: Vec::<&str>::new(), err: Vec::<&str>::new() });
    assert_eq!(Text::lines(&scratch.read("m.toml")), ["clippy = 1.00", "test-run = 0.25"]);
    Ok(())
}

#[test]
fn test_output_that_cannot_be_read_is_reported_and_nothing_is_measured() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let answer = Answer::to(
        &Fake::new(),
        &["libtest-seconds", &scratch.file("m.toml"), "test-run", &scratch.file("absent.txt")],
    );
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("absent.txt"));
    assert_eq!(scratch.read("m.toml"), "");
    Ok(())
}

#[test]
fn test_output_without_a_result_line_is_a_failure_and_nothing_is_measured() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("out.txt", "no result lines here\n")?;
    let answer =
        Answer::to(&Fake::new(), &["libtest-seconds", &scratch.file("m.toml"), "test-run", &scratch.file("out.txt")]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("no libtest result line"));
    assert_eq!(scratch.read("m.toml"), "");
    Ok(())
}

#[test]
fn off_a_pull_request_the_scope_is_the_whole_workspace() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Fake::new();
    let answer = Answer::to(&fake, &Reach::on(&scratch, "push"));
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
    let fake = Fake { changed: "crates/leaf/src/lib.rs\n".to_owned(), ..Fake::new() };
    let answer = Answer::to(&fake, &Reach::on(&scratch, "pull_request"));
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
fn a_history_line_names_the_revision_the_reach_and_every_fan_in_and_is_appended() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("metadata.json", METADATA)?;
    scratch.write("history.tsv", "older\n")?;
    let fake = Fake { changed: "crates/leaf/src/lib.rs\n".to_owned(), ..Fake::new() };
    let arguments = ["history", &scratch.file("metadata.json"), &scratch.file("history.tsv"), "abc123"];
    let answer = Answer::to(&fake, &arguments);
    assert_eq!(answer.exit, ExitCode::SUCCESS);
    assert_eq!(Text::lines(&answer.out), [HISTORY_LINE]);
    Answer::to(&fake, &arguments);
    assert_eq!(Text::lines(&scratch.read("history.tsv")), ["older", HISTORY_LINE, HISTORY_LINE]);
    Ok(())
}

#[test]
fn a_history_needs_git_and_readable_metadata_and_a_writable_file() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("metadata.json", METADATA)?;
    let named = |file: &str| ["history".to_owned(), scratch.file("metadata.json"), scratch.file(file), "r".to_owned()];
    let git = Answer::to(&Fake::refusing("git"), &named("history.tsv"));
    assert_eq!((git.exit, Text::lines(&git.err)), (ExitCode::FAILURE, vec!["cannot run git: boom"]));
    scratch.write("blocker", "")?;
    assert_eq!(Answer::to(&Fake::new(), &named("blocker/history.tsv")).exit, ExitCode::FAILURE);
    let broken = ["history".to_owned(), scratch.file("none.json"), scratch.file("history.tsv"), "r".to_owned()];
    assert!(Answer::to(&Fake::new(), &broken).err.contains("reach failed: cannot read"));
    assert_eq!(scratch.read("history.tsv"), "");
    Ok(())
}

#[test]
fn a_report_is_appended_to_the_summary_under_its_name_or_said_to_be_missing() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("report.txt", "found\n")?;
    scratch.write("summary.md", "# Summary\n")?;
    let arguments = |report: &str| ["summarize".to_owned(), scratch.file(report), scratch.file("summary.md")];
    assert_eq!(Answer::to(&Fake::new(), &arguments("report.txt")).exit, ExitCode::SUCCESS);
    assert_eq!(Answer::to(&Fake::new(), &arguments("absent.txt")).exit, ExitCode::SUCCESS);
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

    fn timed(page: &str) -> Result<Timings, Box<dyn Error>> {
        let scratch = Scratch::new()?;
        scratch.write("page.html", page)?;
        let answer =
            Answer::to(&Fake::new(), &["compile-times", &scratch.file("page.html"), &scratch.file("report.txt")]);
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
    let units: Vec<String> = (0..31).map(|at| Page::unit(&format!("crate{at:02}"), "1.0")).collect();
    let answer = Page::timed(&Page::with(&units.join(",")))?.answer;
    assert_eq!(answer.out.matches(char::from(10)).count(), 31);
    assert!(answer.out.contains("compile time per crate, seconds, longest first, 31.00 in all:\n"));
    Ok(())
}

#[test]
fn a_page_without_readable_unit_data_is_refused() -> Result<(), Box<dyn Error>> {
    for page in ["<html></html>", "const UNIT_DATA = nope;", "const UNIT_DATA = [{\"name\": 3}];"] {
        let Timings { answer, stored } = Page::timed(page)?;
        assert_eq!(answer.exit, ExitCode::FAILURE, "{page}");
        assert!(answer.err.contains("no unit timings in"), "{page}");
        assert_eq!((answer.out.as_str(), stored.as_str()), ("", ""));
    }
    let scratch = Scratch::new()?;
    let missing = Answer::to(&Fake::new(), &["compile-times", &scratch.file("none.html"), &scratch.file("report.txt")]);
    assert_eq!(missing.exit, ExitCode::FAILURE);
    Ok(())
}

#[test]
fn a_program_the_scope_needs_that_cannot_start_is_reported() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let cargo = Answer::to(&Fake::refusing("cargo"), &Reach::on(&scratch, "push"));
    assert_eq!((cargo.exit, Text::lines(&cargo.err)), (ExitCode::FAILURE, vec!["cannot run cargo: boom"]));
    let git = Answer::to(&Fake::refusing("git"), &Reach::on(&scratch, "pull_request"));
    assert_eq!((git.exit, Text::lines(&git.err)), (ExitCode::FAILURE, vec!["cannot run git: boom"]));
    assert_eq!(git.out, "");
    Ok(())
}

#[test]
fn metadata_reach_cannot_use_is_reported_with_its_reason() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Fake { metadata: "not json".to_owned(), ..Fake::new() };
    let answer = Answer::to(&fake, &Reach::on(&scratch, "pull_request"));
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
    let answer = Answer::to(&Fake::new(), &["reach", &scratch.file("blocker/metadata.json"), "push", "", "e", "s"]);
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
        ["compile-times"] => true,
        ["compile-times", "page.html"] => true,
        ["compile-times", "page.html", "report.txt", "extra"] => true,
        ["history"] => true,
        ["history", "metadata.json"] => true,
        ["history", "metadata.json", "history.tsv"] => true,
        ["history", "metadata.json", "history.tsv", "abc", "extra"] => true,
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
        ["mutants-changed"] => true,
        ["mutants-changed", "diff"] => true,
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
    Answer::to(&Fake::new(), &Reach::on(&scratch, "push"));
    assert_eq!(Text::lines(&scratch.read("env")), ["KEPT=1", "BUILD_SCOPE=--workspace"]);
    assert_eq!(Text::lines(&scratch.read("sum")), ["# Summary", "BUILD_SCOPE=--workspace"]);
    Ok(())
}

#[test]
fn an_empty_scope_is_said_in_the_output_and_the_summary_but_not_put_in_the_environment() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let answer = Answer::to(&Fake::new(), &Reach::on(&scratch, "pull_request"));
    assert_eq!(Text::lines(&answer.out), EMPTY_REPORT_AND_NOTES);
    assert_eq!(Text::lines(&scratch.read("env")), EMPTY_REPORT);
    assert_eq!(Text::lines(&scratch.read("sum")), EMPTY_REPORT_AND_NOTES);
    assert_eq!([EMPTY_REPORT.as_slice(), EMPTY_NOTES.as_slice()].concat(), EMPTY_REPORT_AND_NOTES);
    Ok(())
}

#[test]
fn a_scope_with_crates_says_nothing_is_skipped() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Fake { changed: "crates/leaf/src/lib.rs\n".to_owned(), ..Fake::new() };
    assert!(!Answer::to(&fake, &Reach::on(&scratch, "pull_request")).out.contains("skipped"));
    Ok(())
}

#[test]
fn the_tools_are_built_and_copied_to_the_tools_directory() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.directory("built")?;
    ["reach", "lint", "ci"].iter().try_for_each(|name| scratch.write(&format!("built/{name}"), name))?;
    let fake = Fake::new();
    let answer = Answer::to(&fake, &["tools", &scratch.file("built"), &scratch.file("a/b/tools")]);
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: Vec::<&str>::new(), err: Vec::<&str>::new() });
    assert_eq!(*fake.calls.borrow(), [Invocation::new("cargo", [BUILD.as_slice(), &["--bin", "ci"]].concat())]);
    assert_eq!(["reach", "lint", "ci"].map(|name| scratch.read(&format!("a/b/tools/{name}"))), ["reach", "lint", "ci"]);
    Ok(())
}

#[test]
fn tools_that_failed_to_build_are_not_copied() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let answer = Answer::to(&Fake::exiting(101), &["tools", &scratch.file("built"), &scratch.file("tools")]);
    assert_eq!(answer.exit, ExitCode::from(101));
    assert!(!scratch.path().join("tools").exists());
    Ok(())
}

#[test]
fn tools_that_were_not_built_are_reported() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let answer = Answer::to(&Fake::new(), &["tools", &scratch.file("built"), &scratch.file("tools")]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("built/reach"));
    Ok(())
}

#[test]
fn tools_cannot_go_where_a_file_is_and_cargo_must_start() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("blocker", "")?;
    let blocked = Answer::to(&Fake::new(), &["tools", &scratch.file("built"), &scratch.file("blocker/tools")]);
    assert!(blocked.err.contains("blocker/tools"));
    let refused = Answer::to(&Fake::refusing("cargo"), &["tools", &scratch.file("built"), &scratch.file("tools")]);
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
    let fake = Fake { metadata: TOOLS_WORKSPACE.to_owned(), ..Fake::new() };
    let answer = Answer::to(&fake, &["tools-key", &scratch.file("m/metadata.json"), &scratch.file("out")]);
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: Vec::<&str>::new(), err: Vec::<&str>::new() });
    assert_eq!(Text::lines(&scratch.read("out")), [["KEPT=1"].as_slice(), TOOLS_SOURCES.as_slice()].concat());
    Ok(())
}

#[test]
fn a_tools_key_needs_metadata_cargo_can_give_and_reach_can_read() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let arguments = ["tools-key", &scratch.file("metadata.json"), &scratch.file("out")];
    assert_eq!(Text::lines(&Answer::to(&Fake::refusing("cargo"), &arguments).err), ["cannot run cargo: boom"]);
    let broken = Fake { metadata: "not json".to_owned(), ..Fake::new() };
    assert!(Answer::to(&broken, &arguments).err.contains("reach failed: cannot parse cargo metadata"));
    assert!(!scratch.path().join("out").exists());
    Ok(())
}

#[test]
fn a_logged_program_is_run_with_its_log_and_its_exit() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Fake { logged_text: "seen\n".to_owned(), exit: Exit::Failure(FAILED_WITH), ..Fake::new() };
    let answer = Answer::to(&fake, &["logged", &scratch.file("logs/test.txt"), "cargo", "test", "-p", "leaf"]);
    assert_eq!(
        answer.lines(),
        Printed { exit: ExitCode::from(FAILED_WITH.get()), out: Vec::<&str>::new(), err: Vec::<&str>::new() }
    );
    assert_eq!(*fake.calls.borrow(), [Invocation::new("cargo", ["test", "-p", "leaf"])]);
    assert_eq!(*fake.logs.borrow(), [scratch.path().join("logs/test.txt")]);
    assert_eq!(Text::lines(&scratch.read("logs/test.txt")), ["seen"]);
    let refused = Answer::to(&Fake::refusing("cargo"), &["logged", &scratch.file("t.txt"), "cargo"]);
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
        Answer::to(&Fake::exiting(code.0), &["mutants-shard", "3/8", "out"]).exit
    }

    fn changed(code: &Code) -> ExitCode {
        let scratch = Scratch::fresh();
        let arguments = ["mutants-changed", &scratch.file("d.diff"), &scratch.file("out"), "--test-package=leaf"];
        Answer::to(&Fake::exiting(code.0), &arguments).exit
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
    let fake = Fake::new();
    let answer = Answer::to(&fake, &["mutants-shard", "3/8", "/tmp/out"]);
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: Vec::<&str>::new(), err: Vec::<&str>::new() });
    assert_eq!(*fake.calls.borrow(), [Cargo::mutants("3/8", "/tmp/out")]);
    let refused = Answer::to(&Fake::refusing("cargo"), &["mutants-shard", "3/8", "out"]);
    assert_eq!(Text::lines(&refused.err), ["cannot run cargo: boom"]);
}

#[test]
fn changed_lines_are_written_and_tested_in_the_scope() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Fake { changed: "+a line\n".to_owned(), ..Fake::new() };
    let answer = Answer::to(
        &fake,
        &["mutants-changed", &scratch.file("tmp/c.diff"), &scratch.file("out"), "--test-package=a", "--test-package=b"],
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
    let answer = Answer::to(&Fake::new(), &["mutants-changed", &scratch.file("c.diff"), &scratch.file("out"), "-p"]);
    assert_eq!(answer.exit, ExitCode::from(2));
    assert_eq!(Text::lines(&answer.out), ["mutants missed:", "src/a.rs:1: replace f -> bool with true"]);
    Ok(())
}

#[test]
fn an_empty_missed_list_passes() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.directory("out/mutants.out")?;
    scratch.write("out/mutants.out/missed.txt", "")?;
    let answer = Answer::to(&Fake::new(), &["mutants-changed", &scratch.file("c.diff"), &scratch.file("out"), "-p"]);
    assert_eq!((answer.exit, Text::lines(&answer.out)), (ExitCode::SUCCESS, vec!["mutants: none missed"]));
    Ok(())
}

#[test]
fn a_change_that_reaches_no_crate_says_the_mutants_were_skipped() {
    let fake = Fake::new();
    let answer = Answer::to(&fake, &["mutants-changed", "c.diff", "out"]);
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
    let arguments = ["mutants-changed", &scratch.file("c.diff"), &scratch.file("out"), "-p"];
    assert_eq!(Text::lines(&Answer::to(&Fake::refusing("git"), &arguments).err), ["cannot run git: boom"]);
    assert_eq!(Text::lines(&Answer::to(&Fake::refusing("cargo"), &arguments).err), ["cannot run cargo: boom"]);
    scratch.write("blocker", "")?;
    let blocked = Answer::to(&Fake::new(), &["mutants-changed", &scratch.file("blocker/c.diff"), "out", "-p"]);
    assert!(blocked.err.contains("blocker/c.diff"));
    Ok(())
}

#[test]
fn step_times_are_asked_of_github_for_the_job() {
    let fake = Fake::answering(vec![Reply::to("gh api", "clippy: 12\ntest-run: 3\n")]);
    let answer = Answer::to(&fake, &["wall-times", "Studio-Terza/meta-dev", "77", "gates"]);
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
        Text::lines(&Answer::to(&Fake::refusing("gh"), &["wall-times", "r", "1", "gates"]).err),
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
    let fake = Fake::answering(vec![Reply::to("gh issue", "[]")]);
    let answer = Answer::to(&fake, &arguments);
    assert_eq!(answer.exit, ExitCode::SUCCESS);
    assert_eq!(
        *fake.calls.borrow(),
        [
            Gh::search("clippy"),
            Gh::call(&["issue", "create", "--title", "clippy", "--body", CLIPPY_BODY]),
            Gh::search("test-run"),
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
    let fake = Fake::answering(vec![Reply::to("gh issue", listed)]);
    Answer::to(&fake, &arguments);
    assert_eq!(fake.calls.borrow().last(), Some(&Gh::call(&["issue", "edit", "7", "--body", CLIPPY_BODY])));
    Ok(())
}

#[test]
fn stages_named_after_the_files_are_the_only_ones_judged() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let mut arguments = Gh::stage_issues(&scratch, "clippy = 12\n")?;
    arguments.push("clippy".to_owned());
    let fake = Fake::answering(vec![Reply::to("gh issue", "[]")]);
    assert_eq!(Answer::to(&fake, &arguments).exit, ExitCode::SUCCESS);
    assert_eq!(fake.calls.borrow().len(), 2);
    Ok(())
}

#[test]
fn stages_within_their_warn_line_open_nothing() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let arguments = Gh::stage_issues(&scratch, "clippy = 11\nlint = 1\ntest-run = 1\n")?;
    let fake = Fake::new();
    assert_eq!(Answer::to(&fake, &arguments).exit, ExitCode::SUCCESS);
    assert!(fake.calls.borrow().is_empty());
    Ok(())
}

#[test]
fn limits_that_cannot_be_judged_and_issues_that_cannot_be_handled_are_reported() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let arguments = Gh::stage_issues(&scratch, "clippy = 12\nlint = 1\ntest-run = 1\n")?;
    assert_eq!(Text::lines(&Answer::to(&Fake::refusing("gh"), &arguments).err), ["cannot run gh: boom"]);
    let garbled = Fake::answering(vec![Reply::to("gh issue", "not json")]);
    assert!(Answer::to(&garbled, &arguments).err.contains("cannot read the list of issues"));
    let failing = Fake { exit: Exit::Failure(FAILED_WITH), ..Fake::answering(vec![Reply::to("gh issue", "[]")]) };
    assert_eq!(Answer::to(&failing, &arguments).exit, ExitCode::from(FAILED_WITH.get()));
    let unmeasured = Gh::stage_issues(&scratch, "clippy = 12\n")?;
    assert!(Answer::to(&Fake::new(), &unmeasured).err.contains("the limits cannot be judged"));
    Ok(())
}

#[test]
fn files_are_shown_under_their_names_and_absent_ones_are_said_to_be_absent() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("a.txt", "one\n")?;
    let answer = Answer::to(&Fake::new(), &["show", &scratch.file("a.txt"), &scratch.file("b.txt")]);
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
        let fake = Fake::answering(vec![
            Reply::to("git merge-base", "abc123\n"),
            Reply::to("git -c", case.changes),
            Reply::to("gh pr", &[case.kind, "\n", case.labels].concat()),
        ]);
        let scratch = Scratch::fresh();
        let answer = Answer::to(&fake, &["labels", "main", "121", &scratch.file("metadata.json")]);
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
    let fake = Fake::answering(vec![Reply::to("git merge-base", "abc123\n")]);
    let scratch = Scratch::fresh();
    let metadata = scratch.file("metadata.json");
    let answer = Answer::to(&fake, &["labels", "release", "7", &metadata]);
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
        Text::lines(&Answer::to(&Fake::refusing("git"), &["labels", "main", "1", &metadata]).err),
        ["cannot run git: boom"]
    );
    assert_eq!(
        Text::lines(&Answer::to(&Fake::refusing("gh"), &["labels", "main", "1", &metadata]).err),
        ["cannot run gh: boom"]
    );
    assert_eq!(
        Text::lines(&Answer::to(&Fake::refusing("cargo"), &["labels", "main", "1", &metadata]).err),
        ["cannot run cargo: boom"]
    );
}

struct Body;

impl Body {
    const FORMS: &'static str = "write a line spec: none, spec: owner <uuid prefix of 8 or more hex digits>, or spec: <project>#D<n> or <project>#L<n> (several comma-separated); a PR that repairs a red main writes the line repair: main instead\n";
    const REPAIRING: &'static str =
        "repairing main: the body has the line repair: main, so the spec line is not checked\n";

    fn checked(body: &&str) -> Verdict {
        let fake = Fake::answering(vec![Reply::to("gh pr", body)]);
        let answer = Answer::to(&fake, &["spec-line", "172"]);
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
    let fake = Fake::answering(vec![Reply::to("gh pr", "spec: none\n")]);
    let answer = Answer::to(&fake, &["spec-line", "9"]);
    assert_eq!(answer.exit, ExitCode::SUCCESS);
    assert_eq!(*fake.calls.borrow(), [Invocation::new("gh", ["pr", "view", "9", "--json", "body", "--jq", ".body"])]);
    let refused = Answer::to(&Fake::refusing("gh"), &["spec-line", "9"]);
    assert_eq!(Text::lines(&refused.err), ["cannot run gh: boom"]);
    assert!(refused.err.ends_with(char::is_control));
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
    let answer = Answer::to(&Fake::new(), &["shard-summary", &scratch.file("shards"), &scratch.file("sum")]);
    let expected = ["caught: 3", "missed: 2", "timeout: 1", "unviable: 0", "```", "m1", "m2", "t1", "```"];
    assert_eq!(answer.lines(), Printed { exit: ExitCode::SUCCESS, out: expected.to_vec(), err: Vec::<&str>::new() });
    assert_eq!(Text::lines(&scratch.read("sum")), [["before"].as_slice(), expected.as_slice()].concat());
    Ok(())
}

#[test]
fn a_shard_summary_without_shards_is_reported() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let answer = Answer::to(&Fake::new(), &["shard-summary", &scratch.file("absent"), &scratch.file("sum")]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("absent"));
    assert!(!scratch.path().join("sum").exists());
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
    let answer = Answer::to(&Fake::new(), &arguments);
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
    let answer = Answer::to(&Fake::new(), &arguments);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("the mutation matrix failed: "));
    scratch.write("blocker", "")?;
    let blocked = Answer::to(&Fake::new(), &["matrix", "s", "t", &scratch.file("blocker/out"), "sum"]);
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
struct Printed<'answer> {
    exit: ExitCode,
    out: Vec<&'answer str>,
    err: Vec<&'answer str>,
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

mod ports {
    use std::fs;
    use std::io;
    use std::path::Path;

    pub struct Scratch(tempfile::TempDir);

    impl Scratch {
        pub fn new() -> Result<Scratch, io::Error> {
            Ok(Scratch(tempfile::tempdir()?))
        }

        pub fn fresh() -> Scratch {
            Scratch::new().unwrap_or_else(|_| unreachable!("a temporary directory"))
        }

        pub fn placed(path: &Path, text: &str) -> Result<(), io::Error> {
            path.parent().map_or(Ok(()), fs::create_dir_all)?;
            fs::write(path, text)
        }

        pub fn directory(&self, name: &str) -> Result<(), io::Error> {
            fs::create_dir_all(self.0.path().join(name))
        }

        pub fn file(&self, name: &str) -> String {
            self.0.path().join(name).display().to_string()
        }

        pub fn read(&self, name: &str) -> String {
            fs::read_to_string(self.0.path().join(name)).unwrap_or_default()
        }

        pub fn write(&self, name: &str, text: &str) -> Result<(), io::Error> {
            fs::write(self.0.path().join(name), text)
        }

        pub fn path(&self) -> &Path {
            self.0.path()
        }
    }
}

use std::cell::RefCell;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::num::NonZeroU8;
use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

use ci::Cli;
use ci::ports::{Exit, Invocation, Ports, Run};

const METADATA: &str = include_str!("../../reach/tests/metadata/workspace.json");
const LEAF_REPORT: &str = "REACH=1 of 5 crates\nCARGO_SCOPE=-p leaf\nMUTANTS_SCOPE=--test-package=leaf\n";
const TWO_REPORT: &str = "REACH=2 of 5 crates\nCARGO_SCOPE=-p middle -p tested-through\n\
    MUTANTS_SCOPE=--test-package=middle --test-package=tested-through\n";
const EMPTY_REPORT: &str = "REACH=0 of 5 crates\nCARGO_SCOPE=\nMUTANTS_SCOPE=\n";
const ROUNDED_UP: Duration = Duration::from_millis(1_996);
const FAILED_WITH: NonZeroU8 = NonZeroU8::new(3).unwrap();

struct Fake {
    exit: Exit,
    elapsed: Duration,
    changed: String,
    metadata: String,
    failing: Option<&'static str>,
    calls: RefCell<Vec<Invocation>>,
}

impl Fake {
    fn new() -> Fake {
        Fake {
            exit: Exit::Success,
            elapsed: Duration::from_millis(1_500),
            changed: String::new(),
            metadata: METADATA.to_owned(),
            failing: None,
            calls: RefCell::new(Vec::new()),
        }
    }

    fn refusing(program: &'static str) -> Fake {
        Fake { failing: Some(program), ..Fake::new() }
    }

    fn recorded(&self, invocation: &Invocation) -> io::Result<()> {
        self.calls.borrow_mut().push(invocation.clone());
        match self.failing {
            Some(program) if invocation.program() == program => Err(io::Error::other("boom")),
            _ => Ok(()),
        }
    }
}

impl Ports for Fake {
    fn run(&self, invocation: &Invocation) -> io::Result<Run> {
        self.recorded(invocation)?;
        Ok(Run::new(self.exit, self.elapsed))
    }

    fn output(&self, invocation: &Invocation) -> io::Result<String> {
        self.recorded(invocation)?;
        Ok(if invocation.program() == "git" { self.changed.clone() } else { self.metadata.clone() })
    }
}

#[derive(Debug, PartialEq)]
struct Answer {
    exit: ExitCode,
    out: String,
    err: String,
}

impl Answer {
    fn to(fake: &Fake, arguments: &[&str]) -> Answer {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let exit = Cli::answer(arguments.iter().map(OsString::from), fake, &mut out, &mut err);
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

struct Scratch(tempfile::TempDir);

impl Scratch {
    fn new() -> Result<Scratch, io::Error> {
        Ok(Scratch(tempfile::tempdir()?))
    }

    fn file(&self, name: &str) -> String {
        self.0.path().join(name).display().to_string()
    }

    fn read(&self, name: &str) -> String {
        fs::read_to_string(self.0.path().join(name)).unwrap_or_default()
    }

    fn write(&self, name: &str, text: &str) -> Result<(), io::Error> {
        fs::write(self.0.path().join(name), text)
    }

    fn path(&self) -> &Path {
        self.0.path()
    }
}

struct Sample;

impl Sample {
    fn timed_line(elapsed: &Duration) -> String {
        let scratch = Scratch(tempfile::tempdir().unwrap_or_else(|_| unreachable!("a temporary directory")));
        let fake = Fake { elapsed: *elapsed, ..Fake::new() };
        Answer::to(&fake, &["timed", &scratch.file("m.toml"), "clippy", "cargo"]);
        scratch.read("m.toml")
    }

    fn libtest_line(output: &&str) -> String {
        let scratch = Scratch(tempfile::tempdir().unwrap_or_else(|_| unreachable!("a temporary directory")));
        fs::write(scratch.path().join("out.txt"), output).ok();
        Answer::to(&Fake::new(), &["libtest-seconds", &scratch.file("m.toml"), "test-run", &scratch.file("out.txt")]);
        scratch.read("m.toml")
    }

    fn reach_report(changed: &&str) -> String {
        let scratch = Scratch(tempfile::tempdir().unwrap_or_else(|_| unreachable!("a temporary directory")));
        let fake = Fake { changed: (*changed).to_owned(), ..Fake::new() };
        Answer::to(&fake, &["reach", &scratch.file("metadata.json"), "pull_request"]).out
    }
}

laws::table!(
    a_stage_is_recorded_in_seconds_to_two_places,
    Sample::timed_line,
    [
        Duration::from_millis(1_500) => "clippy = 1.50\n",
        ROUNDED_UP => "clippy = 2.00\n",
        Duration::from_millis(4) => "clippy = 0.00\n",
        Duration::from_millis(1_004) => "clippy = 1.00\n",
        Duration::from_millis(1_006) => "clippy = 1.01\n",
        Duration::ZERO => "clippy = 0.00\n",
        Duration::from_secs(125) => "clippy = 125.00\n",
    ]
);

laws::table!(
    the_libtest_seconds_of_every_result_line_are_added,
    Sample::libtest_line,
    [
        "test result: ok. 3 passed; 0 failed; finished in 0.50s\n" => "test-run = 0.50\n",
        "test result: ok. 1 passed; finished in 0.50s\ntest result: ok. 2 passed; finished in 1.25s\n" => "test-run = 1.75\n",
        "running 3 tests\n...\ntest result: ok. finished in 2.00s\nfinished in 9.00s\n" => "test-run = 2.00\n",
        "test result: ok. 1 passed; 0 failed\n" => "test-run = 0.00\n",
        "" => "test-run = 0.00\n",
        "test result: ok. finished in abc\n" => "test-run = 0.00\n",
        "test result: ok. finished in 3s\n" => "test-run = 3.00\n",
        "test result: ok. finished in 0.5\n" => "test-run = 0.50\n",
        "test result: ok. finished in 1s in 2s\n" => "test-run = 3.00\n",
        "  test result: ok. finished in 9s\n" => "test-run = 0.00\n",
        "test results: ok. finished in 4s\n" => "test-run = 4.00\n",
        "test result: ok. finished in\n" => "test-run = 0.00\n",
        "test result: ok. finishedin 5s\n" => "test-run = 0.00\n",
        "test result: ok. in 1.5.5s\n" => "test-run = 0.00\n",
        "test resul: ok. finished in 5s\n" => "test-run = 0.00\n",
        "test result: ok. finished  in\t7s\n" => "test-run = 7.00\n",
        "test result: ok. finished in 0.004s\n" => "test-run = 0.00\n",
    ]
);

laws::table!(
    the_scope_is_what_the_changed_files_reach,
    Sample::reach_report,
    [
        "crates/leaf/src/lib.rs\n" => LEAF_REPORT,
        "crates/middle/src/lib.rs\n" => TWO_REPORT,
        "crates/middle/src/lib.rs\ncrates/middle/src/other.rs\n" => TWO_REPORT,
        "README.md\n" => EMPTY_REPORT,
        "" => EMPTY_REPORT,
    ]
);

#[test]
fn a_timed_stage_runs_the_program_with_its_arguments() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Fake::new();
    let answer = Answer::to(&fake, &["timed", &scratch.file("m.toml"), "clippy", "cargo", "clippy", "--all-targets"]);
    assert_eq!(answer, Answer { exit: ExitCode::SUCCESS, out: String::new(), err: String::new() });
    assert_eq!(*fake.calls.borrow(), [Invocation::new("cargo", ["clippy", "--all-targets"])]);
    let calls = fake.calls.borrow();
    assert_eq!(
        calls.first().map(|call| call.arguments().to_vec()),
        Some(vec![OsString::from("clippy"), OsString::from("--all-targets")])
    );
    assert_eq!(scratch.read("m.toml"), "clippy = 1.50\n");
    Ok(())
}

#[test]
fn a_timed_stage_is_appended_to_what_is_measured() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("m.toml", "fmt = 0.10\n")?;
    Answer::to(&Fake::new(), &["timed", &scratch.file("m.toml"), "clippy", "cargo"]);
    Answer::to(&Fake::new(), &["timed", &scratch.file("m.toml"), "lint", "cargo"]);
    assert_eq!(scratch.read("m.toml"), "fmt = 0.10\nclippy = 1.50\nlint = 1.50\n");
    Ok(())
}

#[test]
fn a_failing_program_fails_the_step_with_its_code_and_is_still_measured() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Fake { exit: Exit::Failure(FAILED_WITH), ..Fake::new() };
    let answer = Answer::to(&fake, &["timed", &scratch.file("m.toml"), "clippy", "cargo"]);
    assert_eq!(answer.exit, ExitCode::from(FAILED_WITH.get()));
    assert_eq!(scratch.read("m.toml"), "clippy = 1.50\n");
    Ok(())
}

#[test]
fn a_program_that_cannot_start_is_reported_and_not_measured() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let answer = Answer::to(&Fake::refusing("cargo"), &["timed", &scratch.file("m.toml"), "clippy", "cargo"]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert_eq!(answer.err, "cannot run cargo: boom\n");
    assert_eq!(scratch.read("m.toml"), "");
    Ok(())
}

#[test]
fn a_measured_file_that_cannot_be_written_is_reported() {
    let answer = Answer::to(&Fake::new(), &["timed", "/no/such/dir/m.toml", "clippy", "cargo"]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("cannot use /no/such/dir/m.toml"));
}

#[test]
fn libtest_seconds_are_appended_under_the_stage_name() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    scratch.write("m.toml", "clippy = 1.00\n")?;
    scratch.write("out.txt", "test result: ok. finished in 0.25s\n")?;
    let answer =
        Answer::to(&Fake::new(), &["libtest-seconds", &scratch.file("m.toml"), "test-run", &scratch.file("out.txt")]);
    assert_eq!(answer, Answer { exit: ExitCode::SUCCESS, out: String::new(), err: String::new() });
    assert_eq!(scratch.read("m.toml"), "clippy = 1.00\ntest-run = 0.25\n");
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
fn off_a_pull_request_the_scope_is_the_whole_workspace() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Fake::new();
    let answer = Answer::to(&fake, &["reach", &scratch.file("metadata.json"), "push"]);
    assert_eq!(
        answer,
        Answer { exit: ExitCode::SUCCESS, out: "CARGO_SCOPE=--workspace\n".to_owned(), err: String::new() }
    );
    assert_eq!(*fake.calls.borrow(), [Invocation::new("cargo", ["metadata", "--no-deps", "--format-version", "1"])]);
    assert_eq!(scratch.read("metadata.json"), METADATA);
    Ok(())
}

#[test]
fn on_a_pull_request_the_changed_files_are_asked_of_git() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Fake { changed: "crates/leaf/src/lib.rs\n".to_owned(), ..Fake::new() };
    let answer = Answer::to(&fake, &["reach", &scratch.file("metadata.json"), "pull_request"]);
    assert_eq!(answer, Answer { exit: ExitCode::SUCCESS, out: LEAF_REPORT.to_owned(), err: String::new() });
    assert_eq!(
        *fake.calls.borrow(),
        [
            Invocation::new("cargo", ["metadata", "--no-deps", "--format-version", "1"]),
            Invocation::new("git", ["diff", "--name-only", "--no-renames", "HEAD^1", "HEAD"]),
        ]
    );
    assert_eq!(scratch.read("metadata.json"), METADATA);
    Ok(())
}

#[test]
fn a_program_the_scope_needs_that_cannot_start_is_reported() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let metadata = scratch.file("metadata.json");
    let cargo = Answer::to(&Fake::refusing("cargo"), &["reach", &metadata, "push"]);
    assert_eq!((cargo.exit, cargo.err.as_str()), (ExitCode::FAILURE, "cannot run cargo: boom\n"));
    let git = Answer::to(&Fake::refusing("git"), &["reach", &metadata, "pull_request"]);
    assert_eq!((git.exit, git.err.as_str()), (ExitCode::FAILURE, "cannot run git: boom\n"));
    assert_eq!(git.out, "");
    Ok(())
}

#[test]
fn metadata_reach_cannot_use_is_reported_with_its_reason() -> Result<(), Box<dyn Error>> {
    let scratch = Scratch::new()?;
    let fake = Fake { metadata: "not json".to_owned(), ..Fake::new() };
    let answer = Answer::to(&fake, &["reach", &scratch.file("metadata.json"), "pull_request"]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("reach failed: cannot parse cargo metadata"));
    assert_eq!(answer.out, "");
    Ok(())
}

#[test]
fn metadata_that_cannot_be_stored_is_reported() {
    let answer = Answer::to(&Fake::new(), &["reach", "/no/such/dir/metadata.json", "push"]);
    assert_eq!(answer.exit, ExitCode::FAILURE);
    assert!(answer.err.contains("cannot use /no/such/dir/metadata.json"));
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
        ["reach", "metadata.json", "push", "extra"] => true,
    ]
);

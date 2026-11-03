use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::process::ExitCode;

use file_ports::Disk;
use gate_limits::Cli;
use ports::Fixture;

const LIMITS: &str = "[test-run]\nwarn = 3\nblock = 5\n[clippy]\nwarn = 10.5\nblock = 20\n";
const LIMIT: &str = "[test-run]\nwarn = 1\nblock = 2\n";

struct Reported {
    exit: ExitCode,
    out: String,
    err: String,
}

struct Run;

impl Run {
    fn cli(limits: &Path, measured: &Path) -> Cli {
        Cli::from_args([limits, measured].map(|path| path.as_os_str().to_owned()).into_iter())
    }

    fn reported(name: &str, limits: &str, measured: &str) -> io::Result<Reported> {
        Self::reported_only(name, limits, measured, &[])
    }

    fn reported_only(name: &str, limits: &str, measured: &str, only: &[&str]) -> io::Result<Reported> {
        let limits = Fixture::stored(&format!("{name}-limits.toml"), limits)?;
        let measured = Fixture::stored(&format!("{name}-measured.toml"), measured)?;
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let named = [limits.as_os_str().to_owned(), measured.as_os_str().to_owned()]
            .into_iter()
            .chain(only.iter().map(OsString::from));
        let exit = Cli::from_args(named).report_to(&Disk, &mut out, &mut err);
        Ok(Reported {
            exit,
            out: String::from_utf8(out).map_err(io::Error::other)?,
            err: String::from_utf8(err).map_err(io::Error::other)?,
        })
    }

    fn judged(name: &str, measured: &str) -> io::Result<Reported> {
        Self::reported(name, LIMITS, measured)
    }
}

#[test]
fn a_run_within_every_warn_level_is_ok_and_does_not_block() -> io::Result<()> {
    let run = Run::judged("all-ok", "test-run = 2.9\nclippy = 10")?;
    assert_eq!(
        Text::lines(&run.out),
        [
            "ok: clippy took 10s, warn above 10.5s, block above 20s",
            "ok: test-run took 2.9s, warn above 3s, block above 5s"
        ]
    );
    assert_eq!(run.exit, ExitCode::SUCCESS);
    Ok(())
}

#[test]
fn above_warn_warns_and_above_block_blocks() -> io::Result<()> {
    let run = Run::judged("warns", "test-run = 4\nclippy = 10")?;
    assert!(run.out.contains("WARN: test-run took 4s, warn above 3s, block above 5s\n"));
    assert_eq!(run.exit, ExitCode::SUCCESS);
    let run = Run::judged("blocks", "test-run = 5.1\nclippy = 1")?;
    assert!(run.out.contains("BLOCK: test-run took 5.1s, warn above 3s, block above 5s\n"));
    assert_eq!(run.exit, ExitCode::FAILURE);
    Ok(())
}

#[test]
fn exactly_on_a_level_is_within_it() -> io::Result<()> {
    let run = Run::judged("on-level", "test-run = 5\nclippy = 10.5")?;
    assert!(run.out.contains("WARN: test-run"));
    assert!(run.out.contains("ok: clippy"));
    assert_eq!(run.exit, ExitCode::SUCCESS);
    let equal = Run::reported("equal-levels", "[test-run]\nwarn = 2\nblock = 2\n", "test-run = 2")?;
    assert_eq!(Text::lines(&equal.out), ["ok: test-run took 2s, warn above 2s, block above 2s"]);
    Ok(())
}

const REFUSALS: [[&str; 4]; 6] = [
    ["unmeasured", LIMITS, "test-run = 1", "was not measured"],
    ["unlimited", LIMITS, "test-run = 1\nclippy = 1\nlint = 1", "no usable limit"],
    ["not-a-number", LIMITS, "test-run = \"fast\"\nclippy = 1", "cannot parse measured"],
    ["warn-above-block", "[test-run]\nwarn = 6\nblock = 5\n", "test-run = 1", "no usable limit"],
    ["no-block", "[test-run]\nwarn = 1\n", "test-run = 1", "cannot parse limits"],
    ["empty", "", "", "no stage has a limit"],
];

#[test]
fn every_unusable_input_fails_closed_and_says_why() -> io::Result<()> {
    for [name, limits, measured, reason] in REFUSALS {
        let run = Run::reported(name, limits, measured)?;
        assert_eq!(run.exit, ExitCode::FAILURE, "{name}");
        assert!(run.err.contains(reason), "{name}: {}", run.err);
    }
    Ok(())
}

#[test]
fn a_run_fails_only_above_its_block_level() -> io::Result<()> {
    let limits = Fixture::stored("within-limits.toml", LIMIT)?;
    let measured = Fixture::stored("within-measured.toml", "test-run = 0.5\n")?;
    assert_eq!(Run::cli(&limits, &measured).report_to(&Disk, &mut Vec::new(), &mut Vec::new()), ExitCode::SUCCESS);
    let limits = Fixture::stored("blocked-limits.toml", LIMIT)?;
    let measured = Fixture::stored("blocked-measured.toml", "test-run = 3\n")?;
    assert_eq!(Run::cli(&limits, &measured).report_to(&Disk, &mut Vec::new(), &mut Vec::new()), ExitCode::FAILURE);
    Ok(())
}

#[test]
fn an_unreadable_measurement_exits_with_failure() -> io::Result<()> {
    let limits = Fixture::stored("unreadable-limits.toml", LIMIT)?;
    let missing = Path::new(env!("CARGO_TARGET_TMPDIR")).join("no-such-measured.toml");
    assert_eq!(Run::cli(&limits, &missing).report_to(&Disk, &mut Vec::new(), &mut Vec::new()), ExitCode::FAILURE);
    Ok(())
}

#[test]
fn only_stages_past_their_warn_level_are_listed_by_name_and_text() -> io::Result<()> {
    let limits = Fixture::stored("past-limits.toml", LIMITS)?;
    let measured = Fixture::stored("past-measured.toml", "test-run = 4\nclippy = 10")?;
    let past = Run::cli(&limits, &measured).past_warn(&Disk).map_err(io::Error::other)?;
    assert_eq!(past.iter().map(|finding| finding.stage().as_ref()).collect::<Vec<&str>>(), ["test-run"]);
    assert_eq!(
        past.iter().map(ToString::to_string).collect::<Vec<String>>(),
        ["WARN: test-run took 4s, warn above 3s, block above 5s"]
    );
    Ok(())
}

#[test]
fn a_blocking_stage_is_past_its_warn_level_too() -> io::Result<()> {
    let limits = Fixture::stored("block-limits.toml", LIMITS)?;
    let measured = Fixture::stored("block-measured.toml", "test-run = 6\nclippy = 10")?;
    let past = Run::cli(&limits, &measured).past_warn(&Disk).map_err(io::Error::other)?;
    assert_eq!(past.iter().map(|finding| finding.blocks().is_some()).collect::<Vec<bool>>(), [true]);
    Ok(())
}

#[test]
fn limits_that_cannot_be_judged_list_nothing() -> io::Result<()> {
    let limits = Fixture::stored("none-limits.toml", LIMITS)?;
    let measured = Fixture::stored("none-measured.toml", "clippy = 10")?;
    assert!(Run::cli(&limits, &measured).past_warn(&Disk).is_err());
    Ok(())
}

#[test]
fn named_stages_are_judged_alone_so_a_job_can_judge_the_stages_it_timed() -> io::Result<()> {
    let run = Run::reported_only("named", LIMITS, "test-run = 4", &["test-run"])?;
    assert_eq!(Text::lines(&run.out), ["WARN: test-run took 4s, warn above 3s, block above 5s"]);
    assert_eq!(run.exit, ExitCode::SUCCESS);
    Ok(())
}

#[test]
fn a_report_only_stage_is_judged_and_printed_but_never_blocks() -> io::Result<()> {
    let only = ["--report-only", "clippy", "absent"];
    let run = Run::reported_only("report-only", LIMITS, "test-run = 4\nclippy = 30", &only)?;
    assert_eq!(
        Text::lines(&run.out),
        [
            "BLOCK: clippy took 30s, warn above 10.5s, block above 20s",
            "WARN: test-run took 4s, warn above 3s, block above 5s"
        ]
    );
    assert_eq!(run.exit, ExitCode::SUCCESS);
    let other = Run::reported_only("report-only-other", LIMITS, "test-run = 6\nclippy = 30", &only)?;
    assert_eq!(other.exit, ExitCode::FAILURE);
    Ok(())
}

#[test]
fn a_named_stage_that_was_not_measured_or_has_no_limit_fails_closed() -> io::Result<()> {
    let unmeasured = Run::reported_only("named-unmeasured", LIMITS, "", &["test-run"])?;
    assert_eq!(unmeasured.exit, ExitCode::FAILURE);
    assert!(unmeasured.err.contains("was not measured"), "{}", unmeasured.err);
    let unknown = Run::reported_only("named-unknown", LIMITS, "test-run = 1", &["test-run", "typo"])?;
    assert_eq!(unknown.exit, ExitCode::FAILURE);
    assert!(unknown.err.contains("stage typo was asked for but has no limit"), "{}", unknown.err);
    let stray = Run::reported_only("named-stray", LIMITS, "test-run = 1\nclippy = 1", &["test-run"])?;
    assert_eq!(stray.exit, ExitCode::FAILURE);
    assert!(stray.err.contains("no usable limit"), "{}", stray.err);
    Ok(())
}

#[test]
fn a_record_blocks_only_above_itself_and_only_a_record_is_lowered() -> io::Result<()> {
    let record = "cold-clippy = 50\n";
    let within = Run::reported("record-within", record, "cold-clippy = 50")?;
    assert_eq!(Text::lines(&within.out), ["ok: cold-clippy took 50s, record 50s, lowered only by ci ratchet-lower"]);
    assert_eq!(within.exit, ExitCode::SUCCESS);
    let above = Run::reported("record-above", record, "cold-clippy = 50.1")?;
    assert_eq!(above.exit, ExitCode::FAILURE);
    let unrecorded = Run::reported("record-missing", record, "cold-clippy = 1\ncold-test-compile = 1")?;
    assert!(unrecorded.err.contains("no usable limit"), "{}", unrecorded.err);
    let levels = Fixture::stored("levels-limits.toml", LIMIT)?;
    let measured = Fixture::stored("levels-measured.toml", "test-run = 0.5\n")?;
    let refused = Run::cli(&levels, &measured).lower(&Disk).map_err(|failure| failure.to_string());
    assert!(refused.is_err_and(|reason| reason.contains("only the owner changes")));
    assert_eq!(Text::lines(&Fixture::text(&levels)?), Text::lines(&LIMIT));
    Ok(())
}

struct Text;

impl Text {
    fn lines(text: &impl AsRef<str>) -> Vec<&str> {
        text_lines::Lines::from(text.as_ref()).collect()
    }
}

#[test]
fn a_record_is_lowered_to_the_measured_time() -> io::Result<()> {
    let record = Fixture::stored("lowered-limits.toml", "cold-clippy = 50\n")?;
    let measured = Fixture::stored("lowered-measured.toml", "cold-clippy = 40")?;
    Run::cli(&record, &measured).lower(&Disk).map_err(io::Error::other)?;
    assert_eq!(Text::lines(&Fixture::text(&record)?), ["cold-clippy = 40"]);
    Ok(())
}

mod ports {
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};

    pub struct Fixture;

    impl Fixture {
        pub fn stored(name: &str, text: &str) -> io::Result<PathBuf> {
            let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
            fs::write(&path, text)?;
            Ok(path)
        }

        pub fn text(path: &Path) -> io::Result<String> {
            fs::read_to_string(path)
        }
    }
}

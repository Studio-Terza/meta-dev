use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use gate_limits::Cli;

const LIMITS: &str = "[test-run]\nwarn = 3\nblock = 5\n[clippy]\nwarn = 10.5\nblock = 20\n";
const LIMIT: &str = "[test-run]\nwarn = 1\nblock = 2\n";

struct Reported {
    exit: ExitCode,
    out: String,
    err: String,
}

struct Run;

impl Run {
    fn fixture(name: &str, text: &str) -> io::Result<PathBuf> {
        let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
        fs::write(&path, text)?;
        Ok(path)
    }

    fn cli(limits: &Path, measured: &Path) -> Cli {
        Cli::from_args([limits, measured].map(|path| path.as_os_str().to_owned()).into_iter())
    }

    fn reported(name: &str, limits: &str, measured: &str) -> io::Result<Reported> {
        Self::reported_only(name, limits, measured, &[])
    }

    fn reported_only(name: &str, limits: &str, measured: &str, only: &[&str]) -> io::Result<Reported> {
        let limits = Self::fixture(&format!("{name}-limits.toml"), limits)?;
        let measured = Self::fixture(&format!("{name}-measured.toml"), measured)?;
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let named = [limits.as_os_str().to_owned(), measured.as_os_str().to_owned()]
            .into_iter()
            .chain(only.iter().map(OsString::from));
        let exit = Cli::from_args(named).report_to(&mut out, &mut err);
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
        run.out,
        "ok: clippy took 10s, warn above 10.5s, block above 20s\nok: test-run took 2.9s, warn above 3s, block above 5s\n"
    );
    assert_eq!(run.exit, ExitCode::SUCCESS);
    Ok(())
}

#[test]
fn above_warn_but_not_above_block_warns_without_blocking() -> io::Result<()> {
    let run = Run::judged("warns", "test-run = 4\nclippy = 10")?;
    assert!(run.out.contains("WARN: test-run took 4s, warn above 3s, block above 5s\n"));
    assert_eq!(run.exit, ExitCode::SUCCESS);
    Ok(())
}

#[test]
fn above_block_blocks() -> io::Result<()> {
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
fn a_run_within_its_levels_exits_successfully() -> io::Result<()> {
    let limits = Run::fixture("within-limits.toml", LIMIT)?;
    let measured = Run::fixture("within-measured.toml", "test-run = 0.5\n")?;
    assert_eq!(Run::cli(&limits, &measured).run(), ExitCode::SUCCESS);
    Ok(())
}

#[test]
fn a_run_above_its_block_level_exits_with_failure() -> io::Result<()> {
    let limits = Run::fixture("blocked-limits.toml", LIMIT)?;
    let measured = Run::fixture("blocked-measured.toml", "test-run = 3\n")?;
    assert_eq!(Run::cli(&limits, &measured).run(), ExitCode::FAILURE);
    Ok(())
}

#[test]
fn an_unreadable_measurement_exits_with_failure() -> io::Result<()> {
    let limits = Run::fixture("unreadable-limits.toml", LIMIT)?;
    let missing = Path::new(env!("CARGO_TARGET_TMPDIR")).join("no-such-measured.toml");
    assert_eq!(Run::cli(&limits, &missing).run(), ExitCode::FAILURE);
    Ok(())
}

#[test]
fn only_stages_past_their_warn_level_are_listed_by_name_and_text() -> io::Result<()> {
    let limits = Run::fixture("past-limits.toml", LIMITS)?;
    let measured = Run::fixture("past-measured.toml", "test-run = 4\nclippy = 10")?;
    let past = Run::cli(&limits, &measured).past_warn().map_err(io::Error::other)?;
    assert_eq!(past.iter().map(|finding| finding.stage().as_ref()).collect::<Vec<&str>>(), ["test-run"]);
    assert_eq!(
        past.iter().map(ToString::to_string).collect::<Vec<String>>(),
        ["WARN: test-run took 4s, warn above 3s, block above 5s"]
    );
    Ok(())
}

#[test]
fn a_blocking_stage_is_past_its_warn_level_too() -> io::Result<()> {
    let limits = Run::fixture("block-limits.toml", LIMITS)?;
    let measured = Run::fixture("block-measured.toml", "test-run = 6\nclippy = 10")?;
    let past = Run::cli(&limits, &measured).past_warn().map_err(io::Error::other)?;
    assert_eq!(past.iter().map(gate_limits::Finding::blocks).collect::<Vec<bool>>(), [true]);
    Ok(())
}

#[test]
fn limits_that_cannot_be_judged_list_nothing() -> io::Result<()> {
    let limits = Run::fixture("none-limits.toml", LIMITS)?;
    let measured = Run::fixture("none-measured.toml", "clippy = 10")?;
    assert!(Run::cli(&limits, &measured).past_warn().is_err());
    Ok(())
}

#[test]
fn named_stages_are_judged_alone_so_a_job_can_judge_the_stages_it_timed() -> io::Result<()> {
    let run = Run::reported_only("named", LIMITS, "test-run = 4", &["test-run"])?;
    assert_eq!(run.out, "WARN: test-run took 4s, warn above 3s, block above 5s\n");
    assert_eq!(run.exit, ExitCode::SUCCESS);
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

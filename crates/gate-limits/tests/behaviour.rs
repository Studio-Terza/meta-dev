use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use gate_limits::{Cli, Failure, Gate};

const LIMITS: &str = "[test-run]\nwarn = 3\nblock = 5\n[clippy]\nwarn = 10.5\nblock = 20\n";

struct Judged;

impl Judged {
    fn text(measured: &str) -> Result<String, Failure> {
        Gate::judge(LIMITS, measured).map(|gate| gate.to_string())
    }

    fn blocks(measured: &str) -> Result<bool, Failure> {
        Gate::judge(LIMITS, measured).map(|gate| gate.blocks())
    }
}

#[test]
fn a_run_within_every_warn_level_is_ok_and_does_not_block() -> Result<(), Failure> {
    assert_eq!(
        Judged::text("test-run = 2.9\nclippy = 10")?,
        "ok: clippy took 10s, warn above 10.5s, block above 20s\nok: test-run took 2.9s, warn above 3s, block above 5s\n"
    );
    assert!(!Judged::blocks("test-run = 2.9\nclippy = 10")?);
    Ok(())
}

#[test]
fn above_warn_but_not_above_block_warns_without_blocking() -> Result<(), Failure> {
    assert!(
        Judged::text("test-run = 4\nclippy = 10")?.contains("WARN: test-run took 4s, warn above 3s, block above 5s\n")
    );
    assert!(!Judged::blocks("test-run = 4\nclippy = 10")?);
    Ok(())
}

#[test]
fn above_block_blocks() -> Result<(), Failure> {
    assert!(
        Judged::text("test-run = 5.1\nclippy = 1")?
            .contains("BLOCK: test-run took 5.1s, warn above 3s, block above 5s\n")
    );
    assert!(Judged::blocks("test-run = 5.1\nclippy = 1")?);
    Ok(())
}

#[test]
fn exactly_on_a_level_is_within_it() -> Result<(), Failure> {
    assert!(Judged::text("test-run = 5\nclippy = 10.5")?.contains("WARN: test-run"));
    assert!(Judged::text("test-run = 5\nclippy = 10.5")?.contains("ok: clippy"));
    assert!(!Judged::blocks("test-run = 5\nclippy = 10.5")?);
    Ok(())
}

#[test]
fn a_limited_stage_that_was_not_measured_fails_closed() {
    assert!(matches!(Judged::blocks("test-run = 1"), Err(Failure::Unmeasured { .. })));
}

#[test]
fn a_measured_stage_without_a_limit_fails_closed() {
    assert!(matches!(Judged::blocks("test-run = 1\nclippy = 1\nlint = 1"), Err(Failure::Unlimited { .. })));
}

#[test]
fn a_measurement_that_is_not_a_number_fails_closed() {
    assert!(matches!(Judged::blocks("test-run = \"fast\"\nclippy = 1"), Err(Failure::Parse { .. })));
}

#[test]
fn a_warn_level_above_the_block_level_is_refused() {
    assert!(matches!(Gate::judge("[test-run]\nwarn = 6\nblock = 5\n", "test-run = 1"), Err(Failure::Unlimited { .. })));
}

#[test]
fn a_limit_without_a_block_level_is_refused() {
    assert!(matches!(Gate::judge("[test-run]\nwarn = 1\n", "test-run = 1"), Err(Failure::Parse { .. })));
}

#[test]
fn an_empty_limits_file_is_refused_rather_than_passing_everything() {
    assert!(matches!(Gate::judge("", ""), Err(Failure::Empty)));
}
const LIMIT: &str = "[test-run]\nwarn = 1\nblock = 2\n";

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

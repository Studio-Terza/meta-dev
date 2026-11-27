use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;

use console_ports::{Complaint, Shown};
use file_ports::{Bytes, Disk, Files, Memory};
use gate_limits::Cli;
use test_failure::Failed;

const GOLDENS: &str = "tests/goldens";
const GOLDEN_EXTENSION: &str = "txt";
const RECORD: &str = "RECORD_GOLDENS";
const OUT: &str = "--- out\n";
const ERR: &str = "--- err\n";
const WRITTEN: &str = "--- written\n";
const NEWLINE: &str = "\n";
const CODES: [ExitCode; 2] = [ExitCode::SUCCESS, ExitCode::FAILURE];
const WORDS: [&str; 2] = ["passed\n", "failed\n"];
const LEVELS: &str = "tests/fixtures/limits/levels.toml";
const LEVEL: &str = "tests/fixtures/limits/level.toml";
const RECORDED: &str = "tests/fixtures/limits/record.toml";

const REPORTS: [[&str; 3]; 23] = [
    ["all-ok", LEVELS, "tests/fixtures/measured/all-ok.toml"],
    ["warns", LEVELS, "tests/fixtures/measured/warns.toml"],
    ["blocks", LEVELS, "tests/fixtures/measured/blocks.toml"],
    ["on-level", LEVELS, "tests/fixtures/measured/on-level.toml"],
    ["equal-levels", "tests/fixtures/limits/equal.toml", "tests/fixtures/measured/two.toml"],
    ["unmeasured", LEVELS, "tests/fixtures/measured/one.toml"],
    ["unlimited", LEVELS, "tests/fixtures/measured/unlimited.toml"],
    ["not-a-number", LEVELS, "tests/fixtures/measured/not-a-number.toml"],
    ["warn-above-block", "tests/fixtures/limits/warn-above-block.toml", "tests/fixtures/measured/one.toml"],
    ["no-block", "tests/fixtures/limits/no-block.toml", "tests/fixtures/measured/one.toml"],
    ["empty", "tests/fixtures/limits/empty.toml", "tests/fixtures/measured/empty.toml"],
    ["within", LEVEL, "tests/fixtures/measured/within.toml"],
    ["blocked", LEVEL, "tests/fixtures/measured/three.toml"],
    ["unreadable", LEVEL, "tests/fixtures/measured/no-such-measured.toml"],
    ["named", LEVELS, "tests/fixtures/measured/four.toml"],
    ["report-only", LEVELS, "tests/fixtures/measured/report-only.toml"],
    ["report-only-other", LEVELS, "tests/fixtures/measured/report-only-other.toml"],
    ["named-unmeasured", LEVELS, "tests/fixtures/measured/empty.toml"],
    ["named-unknown", LEVELS, "tests/fixtures/measured/one.toml"],
    ["named-stray", LEVELS, "tests/fixtures/measured/both-one.toml"],
    ["record-within", RECORDED, "tests/fixtures/measured/cold-50.toml"],
    ["record-above", RECORDED, "tests/fixtures/measured/cold-above.toml"],
    ["record-missing", RECORDED, "tests/fixtures/measured/cold-unrecorded.toml"],
];

const NAMED: &[&str] = &["test-run"];
const REPORTED_ONLY: &[&str] = &["--report-only", "clippy", "absent"];
const STAGES: [&[&str]; 23] = [
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    &[],
    NAMED,
    REPORTED_ONLY,
    REPORTED_ONLY,
    NAMED,
    &["test-run", "typo"],
    NAMED,
    &[],
    &[],
    &[],
];

const PAST: [[&str; 3]; 3] = [
    ["past-warn", LEVELS, "tests/fixtures/measured/warns.toml"],
    ["past-block", LEVELS, "tests/fixtures/measured/block.toml"],
    ["past-none", LEVELS, "tests/fixtures/measured/clippy-only.toml"],
];

const LOWERED: [[&str; 3]; 2] = [
    ["lowered", RECORDED, "tests/fixtures/measured/cold-40.toml"],
    ["lowered-levels", LEVEL, "tests/fixtures/measured/within.toml"],
];

#[test]
fn every_report_prints_its_golden_output() -> Result<(), Failed> {
    let drifted = REPORTS
        .iter()
        .zip(STAGES)
        .map(|([golden, limits, measured], stages)| {
            let arguments = [limits, measured].into_iter().chain(stages).map(OsString::from);
            let answered = Cli::from_args(arguments).report_to(&Disk);
            let code = answered.code();
            let said = CODES.iter().zip(WORDS).filter(|(known, _)| **known == code).map(|(_, word)| word);
            let out: &Shown = answered.as_ref();
            let err: &Complaint = answered.as_ref();
            let shown: String = said.chain([OUT, AsRef::<str>::as_ref(out), ERR, AsRef::<str>::as_ref(err)]).collect();
            let path = Path::new(GOLDENS).join(golden).with_extension(GOLDEN_EXTENSION);
            std::env::var_os(RECORD).map_or(Ok(()), |_| Disk.write(&path, Bytes::new(shown.clone().into_bytes())))?;
            Ok((Disk.read_text(&path)?.as_ref() != shown).then_some(*golden))
        })
        .collect::<Result<Vec<Option<&str>>, Failed>>()?;
    assert_eq!(drifted.into_iter().flatten().collect::<Vec<&str>>(), Vec::<&str>::new());
    Ok(())
}

#[test]
fn only_stages_past_their_warn_level_are_listed() -> Result<(), Failed> {
    let drifted = PAST
        .iter()
        .map(|[golden, limits, measured]| {
            let cli = Cli::from_args([limits, measured].into_iter().map(OsString::from));
            let shown = match cli.past_warn(&Disk) {
                Ok(past) => past.iter().map(|finding| [finding.to_string(), NEWLINE.to_owned()].concat()).collect(),
                Err(failure) => [failure.to_string(), NEWLINE.to_owned()].concat(),
            };
            let path = Path::new(GOLDENS).join(golden).with_extension(GOLDEN_EXTENSION);
            std::env::var_os(RECORD).map_or(Ok(()), |_| Disk.write(&path, Bytes::new(shown.clone().into_bytes())))?;
            Ok((Disk.read_text(&path)?.as_ref() != shown).then_some(*golden))
        })
        .collect::<Result<Vec<Option<&str>>, Failed>>()?;
    assert_eq!(drifted.into_iter().flatten().collect::<Vec<&str>>(), Vec::<&str>::new());
    Ok(())
}

#[test]
fn only_a_record_is_lowered_to_the_measured_time() -> Result<(), Failed> {
    let drifted = LOWERED
        .iter()
        .map(|[golden, limits, measured]| {
            let files: Memory = [limits, measured]
                .into_iter()
                .map(|path| Disk.read(Path::new(path)).map(|bytes| (Path::new(path).to_path_buf(), bytes)))
                .collect::<Result<Memory, _>>()?;
            let cli = Cli::from_args([limits, measured].into_iter().map(OsString::from));
            let lowered = match cli.lower(&files) {
                Ok(record) => record.to_string(),
                Err(failure) => [failure.to_string(), NEWLINE.to_owned()].concat(),
            };
            let shown = [lowered, WRITTEN.to_owned(), files.read_text(Path::new(limits))?.to_string()].concat();
            let path = Path::new(GOLDENS).join(golden).with_extension(GOLDEN_EXTENSION);
            std::env::var_os(RECORD).map_or(Ok(()), |_| Disk.write(&path, Bytes::new(shown.clone().into_bytes())))?;
            Ok((Disk.read_text(&path)?.as_ref() != shown).then_some(*golden))
        })
        .collect::<Result<Vec<Option<&str>>, Failed>>()?;
    assert_eq!(drifted.into_iter().flatten().collect::<Vec<&str>>(), Vec::<&str>::new());
    Ok(())
}

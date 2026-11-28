use std::ffi::OsString;
use std::fmt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use file_ports::{Disk, Files};
use lint::{Cli, Verdict};
use test_failure::{Failed, Golden};
use timer_ports::Monotonic;

const FIXTURES: &str = "tests/.fixtures";
const GOLDENS: &str = "tests/goldens";
const ALL: &str = ".";

#[derive(Debug)]
struct Case<'name> {
    golden: &'name str,
    fixture: &'name str,
    arguments: &'static [&'static str],
}

#[derive(Debug)]
struct Shown {
    verdict: Verdict,
    out: String,
    err: String,
}

#[derive(Debug)]
struct Unwritable;

const CASES: [Case<'static>; 4] = [
    Case { golden: "workspace_timings", fixture: "workspace", arguments: &["--timings", "."] },
    Case { golden: "two_roots", fixture: "workspace", arguments: &["app", "store-ports"] },
    Case { golden: "mounted", fixture: ALL, arguments: &["no_dyn"] },
    Case { golden: "unreadable", fixture: ALL, arguments: &["/nowhere", "bad_manifest/notes"] },
];

#[test]
fn every_fixture_and_case_prints_its_golden_output() -> Result<(), Failed> {
    let fixtures = Disk.entries(&Case::fixture(ALL))?;
    let names: Vec<String> = fixtures.iter().filter_map(|path| path.file_name()?.to_str().map(str::to_owned)).collect();
    let defaults: Vec<Case<'_>> =
        names.iter().map(|name| Case { golden: name, fixture: name, arguments: &[] }).collect();
    defaults.iter().chain(CASES.iter()).try_for_each(Case::checked)
}

#[test]
fn the_binary_exits_by_its_verdict() {
    let exits = [Verdict::Violations, Verdict::Refused].map(ExitCode::from);
    assert_eq!(format!("{exits:?}"), format!("{:?}", [ExitCode::FAILURE, ExitCode::FAILURE]));
    assert_eq!(["/nowhere", "tests/.fixtures/budget_met"].map(ports::Binary::exit), [Some(1_i32), Some(0_i32)]);
}

mod ports {
    use std::process::Command;

    #[derive(Debug)]
    pub(crate) struct Binary;

    impl Binary {
        pub(crate) fn exit(root: &str) -> Option<i32> {
            Command::new(env!("CARGO_BIN_EXE_lint")).arg(root).output().into_iter().find_map(|ran| ran.status.code())
        }
    }
}

#[test]
fn output_that_cannot_be_written_refuses_the_run() -> Result<(), Failed> {
    let mut err = Vec::new();
    let working = Case::fixture("budget_met");
    let verdict = Cli::from_args([]).verdict(&Disk, &working, Monotonic::default(), &mut Unwritable, &mut err);
    assert_eq!((verdict, String::from_utf8(err)?), (Verdict::Refused, "cannot write the output: refused\n".to_owned()));
    Ok(())
}

impl Case<'_> {
    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURES).join(name)
    }

    fn checked(&self) -> Result<(), Failed> {
        let shown = Shown::ran(&Case::fixture(self.fixture), self.arguments)?.to_string();
        Golden::at(Path::new(env!("CARGO_MANIFEST_DIR")), &format!("{GOLDENS}/{}.txt", self.golden)).checked(&shown)
    }
}

impl Shown {
    fn ran(working: &Path, arguments: &[&str]) -> Result<Shown, Failed> {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let given = arguments.iter().map(OsString::from);
        let verdict = Cli::from_args(given).verdict(&Disk, working, Monotonic::default(), &mut out, &mut err);
        Ok(Shown { verdict, out: String::from_utf8(out)?, err: String::from_utf8(err)? })
    }

    fn unnumbered(line: &[char]) -> String {
        let timed = line.first().is_some_and(|first| *first == ' ' || first.is_ascii_digit());
        let label = line.iter().skip_while(|letter| **letter == ' ' || letter.is_ascii_digit());
        if timed { label.collect() } else { line.iter().collect() }
    }
}

impl fmt::Display for Shown {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let letters: Vec<char> = self.out.chars().collect();
        writeln!(formatter, "{:?}\n--- out", self.verdict)?;
        letters
            .chunk_by(|letter, _| *letter != '\n')
            .map(Shown::unnumbered)
            .try_for_each(|line| formatter.write_str(&line))?;
        writeln!(formatter, "--- err")?;
        formatter.write_str(&self.err)
    }
}

impl Write for Unwritable {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("refused"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

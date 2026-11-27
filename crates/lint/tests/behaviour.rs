use std::ffi::OsString;
use std::fmt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use file_ports::{Bytes, Disk, FileFailure, Files, Kind, Memory};
use lint::{Cli, Verdict};
use port_core::timer::Monotonic;
use test_failure::Failed;

const FIXTURES: &str = "tests/fixtures";
const GOLDENS: &str = "tests/goldens";
const STORED_MANIFEST: &str = "manifest.toml";
const MANIFEST: &str = "Cargo.toml";
const RECORD: &str = "RECORD_GOLDENS";
const DEPTH: usize = 5_000;
const BUDGET_MET: &str = "Clean\n--- out\ntest budget: 9 of 50 lines are tests, at most 18% may be\n--- err\n";
const BUDGET_PAST: &str = "Violations\n--- out\n--- err\ntest budget: 10 of 50 lines are tests, at most 18% may be; cut test lines or test through fewer, table-driven cases\n";
const FREE_FN: &str = "free functions are not allowed; make them associated functions or methods:\nsrc/lib.rs:1: fn free; make it an associated function or a method\n";

#[derive(Debug)]
struct Case<'name> {
    golden: &'name str,
    fixture: &'name str,
    mount: &'static str,
    arguments: &'static [&'static str],
}

#[derive(Debug)]
struct Entry {
    path: PathBuf,
    bytes: Bytes,
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
    Case { golden: "workspace_timings", fixture: "workspace", mount: ".", arguments: &["--timings", "."] },
    Case { golden: "two_roots", fixture: "workspace", mount: ".", arguments: &["app", "store-ports"] },
    Case { golden: "mounted", fixture: "no_dyn", mount: "/corpus", arguments: &["/corpus"] },
    Case { golden: "unreadable", fixture: "bad_manifest", mount: "/corpus", arguments: &["/nowhere", "/corpus/notes"] },
];

#[test]
fn every_fixture_and_case_prints_its_golden_output() -> Result<(), Failed> {
    let fixtures = Disk.entries(&Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURES))?;
    let names: Vec<String> = fixtures.iter().filter_map(|path| path.file_name()?.to_str().map(str::to_owned)).collect();
    let defaults: Vec<Case<'_>> =
        names.iter().map(|name| Case { golden: name, fixture: name, mount: ".", arguments: &["."] }).collect();
    let drifted: Vec<String> = defaults
        .iter()
        .chain(CASES.iter())
        .map(Case::drifted)
        .collect::<Result<Vec<Option<String>>, Failed>>()?
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(drifted, Vec::<String>::new());
    Ok(())
}

#[test]
fn no_argument_reads_the_current_directory() -> Result<(), Failed> {
    let files: Memory = [(PathBuf::from("src/lib.rs"), Bytes::new(b"fn free() {}\n".to_vec()))].into_iter().collect();
    let unpackaged = Shown::ran(&files, &[])?;
    assert_eq!((unpackaged.verdict, unpackaged.err.as_str()), (Verdict::Refused, "no Rust sources found under .\n"));
    let manifested: Memory = [
        (PathBuf::from(MANIFEST), Bytes::new(b"[package]\nname = \"p\"\n".to_vec())),
        (PathBuf::from("src/lib.rs"), Bytes::new(b"fn free() {}\n".to_vec())),
    ]
    .into_iter()
    .collect();
    let packaged = Shown::ran(&manifested, &[])?;
    assert_eq!((packaged.verdict, packaged.err.as_str()), (Verdict::Violations, FREE_FN));
    let exits = [Verdict::Violations, Verdict::Refused].map(ExitCode::from);
    assert_eq!(format!("{exits:?}"), format!("{:?}", [ExitCode::FAILURE, ExitCode::FAILURE]));
    assert_eq!(["/nowhere", "."].map(ports::Binary::exit), [Some(1_i32), Some(0_i32)]);
    Ok(())
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
fn deeply_nested_code_is_read_whole() -> Result<(), Failed> {
    let nested = format!(
        "pub struct Deep;\nimpl Deep {{\n    pub fn deep() -> u8 {{\n        {}1{}\n    }}\n}}\n",
        "(".repeat(DEPTH),
        ")".repeat(DEPTH)
    );
    let files: Memory = [
        (PathBuf::from(MANIFEST), Bytes::new(b"[package]\nname = \"deep\"\n".to_vec())),
        (PathBuf::from("src/lib.rs"), Bytes::new(nested.into_bytes())),
    ]
    .into_iter()
    .collect();
    let shown = Shown::ran(&files, &[])?;
    assert_eq!((shown.verdict, shown.err.as_str()), (Verdict::Clean, ""));
    Ok(())
}

#[test]
fn test_lines_past_eighteen_percent_of_all_lines_are_violations() {
    let shown = [9, 10].map(|tested| Shown::ran(&Case::budgeted(tested), &[]).map(|shown| shown.to_string()));
    assert_eq!(shown, [Ok(BUDGET_MET.to_owned()), Ok(BUDGET_PAST.to_owned())]);
}

#[test]
fn output_that_cannot_be_written_refuses_the_run() -> Result<(), Failed> {
    let mut err = Vec::new();
    let verdict = Cli::from_args([]).verdict(&Case::budgeted(9), Monotonic::default(), &mut Unwritable, &mut err);
    assert_eq!((verdict, String::from_utf8(err)?), (Verdict::Refused, "cannot write the output: refused\n".to_owned()));
    Ok(())
}

impl Case<'_> {
    fn budgeted(tested: usize) -> Memory {
        [
            (PathBuf::from(MANIFEST), Bytes::new(b"[package]\nname = \"p\"\n".to_vec())),
            (PathBuf::from("src/lib.rs"), Bytes::new("\n".repeat(50 - tested).into_bytes())),
            (PathBuf::from("tests/t.rs"), Bytes::new("\n".repeat(tested).into_bytes())),
        ]
        .into_iter()
        .collect()
    }

    fn drifted(&self) -> Result<Option<String>, Failed> {
        let files = Case::mounted(self.fixture, self.mount)?;
        let shown = Shown::ran(&files, self.arguments)?.to_string();
        let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join(GOLDENS).join(format!("{}.txt", self.golden));
        std::env::var_os(RECORD).map_or(Ok(()), |_| Disk.write(&golden, Bytes::new(shown.clone().into_bytes())))?;
        let expected = Disk.read_text(&golden)?;
        Ok((shown != expected.as_ref()).then(|| format!("{}:\n{shown}", self.golden)))
    }

    fn mounted(fixture: &str, mount: &str) -> Result<Memory, FileFailure> {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURES).join(fixture);
        let entries = Case::copied(&source, Path::new(mount))?;
        Ok(entries.into_iter().map(|entry| (entry.path, entry.bytes)).collect())
    }

    fn copied(found: &Path, target: &Path) -> Result<Vec<Entry>, FileFailure> {
        match Disk.kind(found) {
            Kind::Directory => {
                let inside = Disk.entries(found)?;
                let copied = inside.iter().map(|inner| {
                    let name = inner.file_name().unwrap_or_default();
                    let renamed = if name == STORED_MANIFEST { MANIFEST.as_ref() } else { name };
                    Case::copied(inner, &target.join(renamed))
                });
                Ok(copied.collect::<Result<Vec<Vec<Entry>>, FileFailure>>()?.into_iter().flatten().collect())
            }
            Kind::File | Kind::Link | Kind::Missing => {
                let path = match target.strip_prefix(".") {
                    Ok(relative) => relative.to_path_buf(),
                    Err(_) => target.to_path_buf(),
                };
                Ok(vec![Entry { path, bytes: Disk.read(found)? }])
            }
        }
    }
}

impl Shown {
    fn ran(files: &Memory, arguments: &[&str]) -> Result<Shown, Failed> {
        let mut out = Vec::new();
        let mut err = Vec::new();
        let given = arguments.iter().map(OsString::from);
        let verdict = Cli::from_args(given).verdict(files, Monotonic::default(), &mut out, &mut err);
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

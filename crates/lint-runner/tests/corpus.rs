use std::error::Error;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cargo_ports::Recorded;
use file_ports::{Bytes, FileFailure, Files, Kind, Memory};
use lint_runner::{Cli, Ports};
use timer_ports::Monotonic;

use text_lines::Lines;

const FIXTURES: &str = "tests/fixtures";
const STORED_MANIFEST: &str = "manifest.toml";
const MANIFEST: &str = "Cargo.toml";
const METADATA: &str = "metadata.json";
const SITE_END: &str = ": ";
const LINE_START: &str = ":";
const FIX_START: &str = "; ";
const NO_DEPENDENCIES: &str = "{\"workspace_root\": \"/\", \"packages\": []}";

#[test]
fn every_lint_prints_its_golden_over_its_fixture_and_every_violation_names_its_fix() -> Result<(), Box<dyn Error>> {
    let corpus = golden::fixtures!(FIXTURES)?;
    let fixtures = corpus.entries(&Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURES))?;
    assert_eq!(fixtures.len(), Corpus::lints());
    let outputs = fixtures
        .iter()
        .map(|fixture| Ok((fixture.as_path(), Corpus::output(&corpus, fixture)?)))
        .collect::<Result<Vec<(&Path, String)>, Box<dyn Error>>>()?;
    let drifted: Vec<String> = outputs.iter().flat_map(|(fixture, output)| Corpus::verify(fixture, output)).collect();
    assert_eq!(drifted, Vec::<String>::new());
    let lines: Vec<String> =
        outputs.iter().flat_map(|(_, output)| output.parse::<Lines>().into_iter().flatten()).collect();
    let violations: Vec<Violation<'_>> = lines.iter().map(String::as_str).filter_map(Corpus::violation).collect();
    let unfixed: Vec<(&str, usize)> = violations
        .iter()
        .filter(|violation| Corpus::fix_named(violation.said).is_none())
        .map(|violation| (violation.path, violation.at))
        .collect();
    assert_ne!(violations.len(), 0);
    assert_eq!(unfixed, Vec::<(&str, usize)>::new());
    Ok(())
}

struct Corpus;

struct Copied {
    path: PathBuf,
    bytes: Bytes,
}

struct Violation<'line> {
    path: &'line str,
    at: usize,
    said: &'line str,
}

impl Corpus {
    fn renamed(path: &Path) -> &OsStr {
        let name = path.file_name().unwrap_or_default();
        Some(name).filter(|candidate| *candidate == STORED_MANIFEST).map_or(name, |_| MANIFEST.as_ref())
    }

    fn copied(corpus: &Memory, found: &Path, target: &Path) -> Result<Vec<Copied>, FileFailure> {
        match corpus.kind(found) {
            Kind::Directory => Self::copied_directory(corpus, found, target),
            Kind::File | Kind::Link | Kind::Missing => {
                Ok(vec![Copied { path: target.to_path_buf(), bytes: corpus.read(found)? }])
            }
        }
    }

    fn copied_directory(corpus: &Memory, found: &Path, target: &Path) -> Result<Vec<Copied>, FileFailure> {
        let inside = corpus
            .entries(found)?
            .iter()
            .map(|inner| Self::copied(corpus, inner, &target.join(Self::renamed(inner))))
            .collect::<Result<Vec<Vec<Copied>>, FileFailure>>()?;
        Ok(inside.into_iter().flatten().collect())
    }

    fn output(corpus: &Memory, fixture: &Path) -> Result<String, Box<dyn Error>> {
        let root: Memory = corpus
            .entries(fixture)?
            .iter()
            .filter(|found| found.file_name() != Some(METADATA.as_ref()))
            .map(|found| Self::copied(corpus, found, Path::new(Self::renamed(found))))
            .collect::<Result<Vec<Vec<Copied>>, FileFailure>>()?
            .into_iter()
            .flatten()
            .map(|copied| (copied.path, copied.bytes))
            .collect();
        let described = corpus.read(&fixture.join(METADATA)).unwrap_or_else(|_| Bytes::from(NO_DEPENDENCIES));
        let (mut printed, mut failed) = (Vec::new(), Vec::new());
        let exit = Cli::from_args(std::iter::empty()).report_to(
            Ports::new(&root, [(PathBuf::from(""), described)].into_iter().collect::<Recorded>(), Monotonic::default()),
            &mut printed,
            &mut failed,
        );
        let verdict = Some(exit).filter(|code| *code == ExitCode::SUCCESS).map_or("fails\n", |_| "passes\n");
        Ok(format!("{}{}{verdict}", String::from_utf8(printed)?, String::from_utf8(failed)?))
    }

    fn lints() -> usize {
        let runner = include_str!("../src/cli/mode/lints.rs");
        [
            "::report(readout.files())",
            "flows.report::<",
            "::dirty(&system)",
            "::of_names(extraction.names())",
            "ModelLint>::report",
            "WorkspaceLint>::report",
        ]
        .iter()
        .map(|call| runner.matches(call).count())
        .sum::<usize>()
            + include_str!("../src/cli/mode.rs").matches("::report(files)").count()
    }

    fn violation(line: &str) -> Option<Violation<'_>> {
        let (end, _) = line.match_indices(SITE_END).next()?;
        let site = line.get(..end)?;
        let (colon, _) = site.rmatch_indices(LINE_START).next()?;
        Some(Violation {
            path: site.get(..colon)?,
            at: site.get(colon + LINE_START.len()..)?.parse().ok()?,
            said: line.get(end + SITE_END.len()..)?,
        })
    }

    fn fix_named(line: &str) -> Option<&str> {
        line.rmatch_indices(FIX_START)
            .next()
            .and_then(|(at, _)| line.get(at + FIX_START.len()..))
            .filter(|fix| fix.chars().filter(|letter| letter.is_whitespace()).count() < fix.chars().count())
    }

    fn verify(fixture: &Path, output: &str) -> Vec<String> {
        let name = fixture.file_name().unwrap_or_default().to_string_lossy();
        golden::verdict!(format!("lints/{name}.txt"), output)
            .err()
            .into_iter()
            .map(|failure| failure.to_string())
            .collect()
    }
}

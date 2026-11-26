use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use test_failure::Failed;

use file_ports::{Bytes, Disk, FileFailure, Files, Kind, Memory};
use lint_runner::{Cli, Ports};
use timer_ports::Monotonic;

use crate::ignored::Ignored;
use invariants::Ignoring;
use text_lines::Lines;

const FIXTURES: &str = "tests/fixtures";
const EXPECTED: &str = "tests/goldens/lints";
const STORED_MANIFEST: &str = "manifest.toml";
const MANIFEST: &str = "Cargo.toml";
const SITE_END: &str = ": ";
const LINE_START: &str = ":";
const FIX_START: &str = "; ";

#[test]
fn every_lint_prints_its_expected_output_over_its_fixture_and_every_violation_names_its_fix() -> Result<(), Failed> {
    let corpus = Disk;
    let fixtures = corpus.entries(&Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURES))?;
    assert_eq!(fixtures.len(), Corpus::lints());
    let outputs = fixtures
        .iter()
        .map(|fixture| Ok((fixture.as_path(), Corpus::output(&corpus, fixture)?)))
        .collect::<Result<Vec<(&Path, String)>, Failed>>()?;
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

pub(crate) struct Corpus;

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

    fn copied(corpus: &impl Files, found: &Path, target: &Path) -> Result<Vec<Copied>, FileFailure> {
        match corpus.kind(found) {
            Kind::Directory => Self::copied_directory(corpus, found, target),
            Kind::File | Kind::Link | Kind::Missing => {
                Ok(vec![Copied { path: target.to_path_buf(), bytes: corpus.read(found)? }])
            }
        }
    }

    fn copied_directory(corpus: &impl Files, found: &Path, target: &Path) -> Result<Vec<Copied>, FileFailure> {
        let inside = corpus
            .entries(found)?
            .iter()
            .map(|inner| Self::copied(corpus, inner, &target.join(Self::renamed(inner))))
            .collect::<Result<Vec<Vec<Copied>>, FileFailure>>()?;
        Ok(inside.into_iter().flatten().collect())
    }

    fn output(corpus: &impl Files, fixture: &Path) -> Result<String, Failed> {
        let root: Memory = corpus
            .entries(fixture)?
            .iter()
            .map(|found| Self::copied(corpus, found, Path::new(Self::renamed(found))))
            .collect::<Result<Vec<Vec<Copied>>, FileFailure>>()?
            .into_iter()
            .flatten()
            .map(|copied| (copied.path, copied.bytes))
            .collect();
        let (mut printed, mut failed) = (Vec::new(), Vec::new());
        let exit = Cli::from_args(std::iter::empty()).report_to(
            Ports::new(&root, Monotonic::default()),
            &mut printed,
            &mut failed,
        );
        let verdict = Some(exit).filter(|code| *code == ExitCode::SUCCESS).map_or("fails\n", |_| "passes\n");
        Ok(format!("{}{}{verdict}", String::from_utf8(printed)?, String::from_utf8(failed)?))
    }

    const REGISTERED: [&str; 4] = ["    Filed<", "    Swept<", "    Judged<", "    Ruled<"];

    fn lints() -> usize {
        let runner = include_str!("../src/cli/mode/lints.rs");
        Self::REGISTERED.iter().map(|call| runner.matches(call).count()).sum::<usize>()
    }

    fn violation(line: &str) -> Option<Violation<'_>> {
        let (end, _) = line.match_indices(SITE_END).next()?;
        let site = line.get(..end)?;
        let (colon, _) = site.rmatch_indices(LINE_START).next()?;
        Some(Violation {
            path: site.get(..colon)?,
            at: site.get(colon + LINE_START.len()..)?.parse().ignoring(Ignored::Unparsed)?,
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
        let expected = Path::new(env!("CARGO_MANIFEST_DIR")).join(EXPECTED).join(format!("{name}.txt"));
        let wanted = Disk.read_text(&expected).ignoring(Ignored::Unreadable).map(|text| text.to_string());
        (wanted.as_deref() != Some(output))
            .then_some(name)
            .map(|drifted| format!("{drifted} printed:\n{output}"))
            .into_iter()
            .collect()
    }
}

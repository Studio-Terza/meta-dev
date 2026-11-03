use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;
use std::process::ExitCode;

use file_ports::{FileFailure, Files, Kind, Memory};
use lint_runner::Cli;
use text_lines::Lines;

const FIXTURES: &str = "tests/fixtures";
const STORED_MANIFEST: &str = "manifest.toml";
const MANIFEST: &str = "Cargo.toml";
const SITE_END: &str = ": ";
const FIX_START: &str = "; ";

#[test]
fn every_lint_prints_its_golden_over_its_fixture_and_every_violation_names_its_fix() -> Result<(), Box<dyn Error>> {
    let corpus = golden::fixtures!(FIXTURES)?;
    let fixtures = corpus.entries(&Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURES))?;
    assert_eq!(fixtures.len(), Corpus::lints());
    let outputs = fixtures
        .iter()
        .map(|fixture| Ok((fixture.as_path(), Corpus::output(&corpus, fixture)?)))
        .collect::<Result<Vec<(&Path, String)>, Box<dyn Error>>>()?;
    let differences: Vec<String> =
        outputs.iter().filter_map(|(fixture, output)| Corpus::difference(fixture, output)).collect();
    assert_eq!(differences, Vec::<String>::new());
    let printed: Vec<&str> = outputs
        .iter()
        .flat_map(|(_, output)| Lines::from(output.as_str()))
        .filter(|line| Corpus::is_violation(line))
        .collect();
    let unfixed: Vec<&str> = printed.iter().copied().filter(|line| Corpus::fix_named(line).is_none()).collect();
    assert_ne!(printed, Vec::<&str>::new());
    assert_eq!(unfixed, Vec::<&str>::new());
    Ok(())
}

struct Corpus;

impl Corpus {
    fn renamed(path: &Path) -> &OsStr {
        let name = path.file_name().unwrap_or_default();
        Some(name).filter(|name| *name == STORED_MANIFEST).map_or(name, |_| MANIFEST.as_ref())
    }

    fn copied(corpus: &Memory, found: &Path, target: &Path, into: &Memory) -> Result<(), FileFailure> {
        match corpus.kind(found) {
            Kind::Directory => Self::copied_directory(corpus, found, target, into),
            _ => into.write(target, corpus.read(found)?),
        }
    }

    fn copied_directory(corpus: &Memory, found: &Path, target: &Path, into: &Memory) -> Result<(), FileFailure> {
        into.create_directories(target)?;
        corpus
            .entries(found)?
            .iter()
            .try_for_each(|inner| Self::copied(corpus, inner, &target.join(Self::renamed(inner)), into))
    }

    fn output(corpus: &Memory, fixture: &Path) -> Result<String, Box<dyn Error>> {
        let root = Memory::default();
        corpus
            .entries(fixture)?
            .iter()
            .try_for_each(|found| Self::copied(corpus, found, Path::new(Self::renamed(found)), &root))?;
        let (mut printed, mut failed) = (Vec::new(), Vec::new());
        let exit = Cli::from_args(std::iter::empty()).report_to(&root, &mut printed, &mut failed);
        let verdict = Some(exit).filter(|exit| *exit == ExitCode::SUCCESS).map_or("fails\n", |_| "passes\n");
        Ok(format!("{}{}{verdict}", String::from_utf8(printed)?, String::from_utf8(failed)?))
    }

    fn lints() -> usize {
        [include_str!("../src/lints.rs"), include_str!("../src/mode.rs")]
            .iter()
            .flat_map(|source| ["::report(files)", "flows.report::<"].map(|call| source.matches(call).count()))
            .sum()
    }

    fn is_violation(line: &str) -> bool {
        line.match_indices(SITE_END)
            .next()
            .and_then(|(at, _)| line.get(..at))
            .and_then(|site| site.chars().last())
            .is_some_and(|last| last.is_ascii_digit())
    }

    fn fix_named(line: &str) -> Option<&str> {
        line.rmatch_indices(FIX_START)
            .next()
            .and_then(|(at, _)| line.get(at + FIX_START.len()..))
            .filter(|fix| fix.chars().filter(|letter| letter.is_whitespace()).count() < fix.chars().count())
    }

    fn difference(fixture: &Path, output: &str) -> Option<String> {
        let name = fixture.file_name().unwrap_or_default().to_string_lossy();
        golden::golden!(format!("lints/{name}.txt"), output).err().map(|failure| format!("{failure:?}"))
    }
}

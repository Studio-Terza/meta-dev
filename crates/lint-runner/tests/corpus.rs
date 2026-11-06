use std::error::Error;
use std::ffi::OsStr;
use std::iter;
use std::path::{Path, PathBuf};
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
    let drifted: Vec<String> = outputs.iter().flat_map(|(fixture, output)| Corpus::verify(fixture, output)).collect();
    assert_eq!(drifted, Vec::<String>::new());
    let printed: Vec<&str> = outputs
        .iter()
        .flat_map(|(_, output)| Lines::from(output.as_str()))
        .filter(|line| {
            let site = line.match_indices(SITE_END).next().and_then(|(at, _)| line.get(..at));
            matches!(site.and_then(|site| site.chars().last()), Some(last) if last.is_ascii_digit())
        })
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

    fn copied(corpus: &Memory, found: &Path, target: &Path, into: &Memory) -> Result<Vec<PathBuf>, FileFailure> {
        match corpus.kind(found) {
            Kind::Directory => Self::copied_directory(corpus, found, target, into),
            Kind::File | Kind::Link | Kind::Missing => Self::copied_file(corpus, found, target, into),
        }
    }

    fn copied_file(corpus: &Memory, found: &Path, target: &Path, into: &Memory) -> Result<Vec<PathBuf>, FileFailure> {
        into.write(target, corpus.read(found)?)?;
        Ok(vec![target.to_path_buf()])
    }

    fn copied_directory(
        corpus: &Memory,
        found: &Path,
        target: &Path,
        into: &Memory,
    ) -> Result<Vec<PathBuf>, FileFailure> {
        into.create_directories(target)?;
        let inside = corpus
            .entries(found)?
            .iter()
            .map(|inner| Self::copied(corpus, inner, &target.join(Self::renamed(inner)), into))
            .collect::<Result<Vec<Vec<PathBuf>>, FileFailure>>()?;
        Ok(iter::once(target.to_path_buf()).chain(inside.into_iter().flatten()).collect())
    }

    fn output(corpus: &Memory, fixture: &Path) -> Result<String, Box<dyn Error>> {
        let root = Memory::default();
        let written: Vec<PathBuf> = corpus
            .entries(fixture)?
            .iter()
            .map(|found| Self::copied(corpus, found, Path::new(Self::renamed(found)), &root))
            .collect::<Result<Vec<Vec<PathBuf>>, FileFailure>>()?
            .into_iter()
            .flatten()
            .collect();
        assert_ne!(written, Vec::<PathBuf>::new());
        assert!(written.iter().all(|path| root.kind(path) != Kind::Missing));
        let (mut printed, mut failed) = (Vec::new(), Vec::new());
        let exit = Cli::from_args(std::iter::empty()).report_to(&root, &mut printed, &mut failed);
        let verdict = Some(exit).filter(|exit| *exit == ExitCode::SUCCESS).map_or("fails\n", |_| "passes\n");
        Ok(format!("{}{}{verdict}", String::from_utf8(printed)?, String::from_utf8(failed)?))
    }

    fn lints() -> usize {
        let runner = include_str!("../src/cli/mode/lints.rs");
        ["::report(readout.files())", "flows.report::<", "::dirty(&system)", "ModelLint>::report"]
            .iter()
            .map(|call| runner.matches(call).count())
            .sum::<usize>()
            + include_str!("../src/cli/mode.rs").matches("::report(files)").count()
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

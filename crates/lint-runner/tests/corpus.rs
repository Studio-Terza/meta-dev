use std::error::Error;
use std::path::Path;

use golden::{Golden, Output};
use ports::Corpus;
use text_lines::Lines;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
const STORED_MANIFEST: &str = "manifest.toml";
const MANIFEST: &str = "Cargo.toml";
const SITE_END: &str = ": ";
const FIX_START: &str = "; ";

#[test]
fn every_lint_prints_its_golden_over_its_fixture_and_every_violation_names_its_fix() -> Result<(), Box<dyn Error>> {
    let fixtures = Corpus::fixtures()?;
    assert_eq!(fixtures.len(), Corpus::lints()?);
    let outputs = fixtures
        .iter()
        .map(|fixture| Ok((fixture.as_path(), Corpus::output(fixture)?)))
        .collect::<Result<Vec<(&Path, String)>, Box<dyn Error>>>()?;
    let differences: Vec<String> = outputs
        .iter()
        .map(|(fixture, output)| Corpus::difference(fixture, output))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(differences, Vec::<String>::new());
    let printed: Vec<&str> = outputs
        .iter()
        .flat_map(|(_, output)| Lines::from(output.as_str()))
        .filter(|line| Corpus::is_violation(line))
        .collect();
    let unfixed: Vec<&str> = printed.iter().copied().filter(|line| !Corpus::names_a_fix(line)).collect();
    assert_ne!(printed, Vec::<&str>::new());
    assert_eq!(unfixed, Vec::<&str>::new());
    Ok(())
}

impl Corpus {
    fn is_violation(line: &str) -> bool {
        line.match_indices(SITE_END)
            .next()
            .and_then(|(at, _)| line.get(..at))
            .and_then(|site| site.chars().last())
            .is_some_and(|last| last.is_ascii_digit())
    }

    fn names_a_fix(line: &str) -> bool {
        line.rmatch_indices(FIX_START)
            .next()
            .and_then(|(at, _)| line.get(at + FIX_START.len()..))
            .is_some_and(|fix| fix.chars().any(|letter| !letter.is_whitespace()))
    }

    fn difference(fixture: &Path, output: &str) -> Result<Option<String>, Box<dyn Error>> {
        let name = fixture.file_name().unwrap_or_default().to_string_lossy();
        let golden = Golden::in_running_package(format!("lints/{name}.txt"))?;
        Ok(golden.verify(Output::new(output)).err().map(|failure| format!("{failure:?}")))
    }
}

mod ports {
    use std::env;
    use std::error::Error;
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::process::ExitCode;

    use lint_runner::Cli;

    use super::{FIXTURES, MANIFEST, STORED_MANIFEST};

    pub struct Corpus;

    impl Corpus {
        pub fn fixtures() -> io::Result<Vec<PathBuf>> {
            fs::read_dir(FIXTURES)?.map(|entry| Ok(entry?.path())).collect()
        }

        fn copied(from: &Path, to: &Path) -> io::Result<()> {
            fs::create_dir_all(to)?;
            for entry in fs::read_dir(from)? {
                let found = entry?.path();
                let name = found.file_name().unwrap_or_default();
                let target = to.join(if name == STORED_MANIFEST { MANIFEST.as_ref() } else { name });
                if found.is_dir() {
                    Self::copied(&found, &target)?;
                } else {
                    fs::copy(&found, &target)?;
                }
            }
            Ok(())
        }

        pub fn output(fixture: &Path) -> Result<String, Box<dyn Error>> {
            let root = tempfile::tempdir()?;
            Self::copied(fixture, root.path())?;
            env::set_current_dir(root.path())?;
            let (mut printed, mut failed) = (Vec::new(), Vec::new());
            let exit = Cli::from_args(std::iter::empty()).report_to(&file_ports::Disk, &mut printed, &mut failed);
            let verdict = if exit == ExitCode::SUCCESS { "passes\n" } else { "fails\n" };
            Ok(format!("{}{}{verdict}", String::from_utf8(printed)?, String::from_utf8(failed)?))
        }

        pub fn lints() -> io::Result<usize> {
            let listed = |file: &str| -> io::Result<usize> {
                Ok(fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(file))?
                    .matches("::report(files)")
                    .count())
            };
            Ok(listed("src/lints.rs")? + listed("src/cli.rs")?)
        }
    }
}

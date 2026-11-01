use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use golden::Golden;
use lint_runner::Lints;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
const STORED_MANIFEST: &str = "manifest.toml";
const MANIFEST: &str = "Cargo.toml";

#[test]
fn every_lint_prints_its_golden_over_its_fixture() -> Result<(), Box<dyn Error>> {
    let fixtures = Corpus::fixtures()?;
    assert_eq!(fixtures.len(), Lints::reports(&[]).len());
    let differences: Vec<String> = fixtures
        .iter()
        .map(|fixture| Corpus::difference(fixture))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(differences, Vec::<String>::new());
    Ok(())
}

struct Corpus;

impl Corpus {
    fn fixtures() -> io::Result<Vec<PathBuf>> {
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

    fn output(fixture: &Path) -> Result<String, Box<dyn Error>> {
        let root = tempfile::tempdir()?;
        Self::copied(fixture, root.path())?;
        env::set_current_dir(root.path())?;
        Ok(Lints::check_root(Path::new("."))?.to_string())
    }

    fn difference(fixture: &Path) -> Result<Option<String>, Box<dyn Error>> {
        let name = fixture.file_name().unwrap_or_default().to_string_lossy();
        let golden = Golden::in_running_package(format!("lints/{name}.txt"))?;
        Ok(golden.verify(&Self::output(fixture)?).err().map(|failure| format!("{failure:?}")))
    }
}

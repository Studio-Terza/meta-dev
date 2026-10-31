use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::failure::Failure;
use crate::tree::Tree;

#[derive(Debug)]
pub struct Roots;

impl Roots {
    const TARGET_DIRECTORIES: [&'static str; 4] = ["src/bin", "tests", "examples", "benches"];

    pub fn under(tree: &Tree) -> Result<BTreeSet<PathBuf>, Failure> {
        let mut candidates = Self::directories(tree.canonical())?;
        candidates.extend(tree.canonical().ancestors().skip(1).map(Path::to_path_buf));
        let mut roots = BTreeSet::new();
        for candidate in candidates {
            roots.extend(Self::targets(&candidate)?.into_iter().filter(|file| file.starts_with(tree.canonical())));
        }
        Ok(roots)
    }

    fn entries(directory: &Path) -> Result<Vec<PathBuf>, Failure> {
        let failed = |source: io::Error| Failure::Io { path: directory.to_path_buf(), source };
        let mut entries = fs::read_dir(directory)
            .map_err(failed)?
            .map(|entry| entry.map(|found| found.path()).map_err(failed))
            .collect::<Result<Vec<PathBuf>, Failure>>()?;
        entries.sort();
        Ok(entries)
    }

    fn is_real_directory(path: &Path) -> bool {
        fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir())
    }

    fn is_build_directory(path: &Path) -> bool {
        path.file_name() == Some(OsStr::new("target"))
            && path.parent().is_some_and(|parent| parent.join("Cargo.toml").is_file())
    }

    fn directories(directory: &Path) -> Result<Vec<PathBuf>, Failure> {
        let mut found = vec![directory.to_path_buf()];
        for entry in Self::entries(directory)? {
            if Self::is_real_directory(&entry) && !Self::is_build_directory(&entry) {
                found.extend(Self::directories(&entry)?);
            }
        }
        Ok(found)
    }

    fn targets(package: &Path) -> Result<Vec<PathBuf>, Failure> {
        let mut found: Vec<PathBuf> =
            [package.join("build.rs"), package.join("src/lib.rs"), package.join("src/main.rs")]
                .into_iter()
                .filter(|file| file.is_file())
                .collect();
        for directory in Self::TARGET_DIRECTORIES {
            found.extend(Self::crate_roots(&package.join(directory))?);
        }
        Ok(found)
    }

    fn crate_roots(directory: &Path) -> Result<Vec<PathBuf>, Failure> {
        if !Self::is_real_directory(directory) {
            return Ok(Vec::new());
        }
        Ok(Self::entries(directory)?
            .into_iter()
            .filter_map(|entry| {
                if Self::is_real_directory(&entry) {
                    Some(entry.join("main.rs")).filter(|main| main.is_file())
                } else {
                    Some(entry).filter(|file| file.extension() == Some(OsStr::new("rs")))
                }
            })
            .collect())
    }
}

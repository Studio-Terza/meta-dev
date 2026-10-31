use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::failure::Failure;
use crate::source_file::SourceFile;

#[derive(Debug)]
pub struct SourceFiles;

impl SourceFiles {
    pub fn read(root: &Path) -> Result<Vec<SourceFile>, Failure> {
        Self::rust_files(root)?
            .into_iter()
            .map(|file| {
                let text = fs::read_to_string(&file).map_err(|source| Failure::Io { path: file.clone(), source })?;
                SourceFile::parse(file.strip_prefix(root).unwrap_or(&file), text)
            })
            .collect()
    }

    fn rust_files(directory: &Path) -> Result<Vec<PathBuf>, Failure> {
        let io_failure = |source: io::Error| Failure::Io { path: directory.to_path_buf(), source };
        let mut entries = fs::read_dir(directory)
            .map_err(io_failure)?
            .map(|entry| entry.map(|found| found.path()).map_err(io_failure))
            .collect::<Result<Vec<PathBuf>, Failure>>()?;
        entries.sort();
        let nested = entries
            .into_iter()
            .map(|path| {
                if path.is_dir() {
                    if Self::is_build_or_hidden(&path) { Ok(Vec::new()) } else { Self::rust_files(&path) }
                } else if path.extension() == Some(OsStr::new("rs")) {
                    Ok(vec![path])
                } else {
                    Ok(Vec::new())
                }
            })
            .collect::<Result<Vec<Vec<PathBuf>>, Failure>>()?;
        Ok(nested.into_iter().flatten().collect())
    }

    fn is_build_or_hidden(directory: &Path) -> bool {
        directory.file_name().is_some_and(|name| name == "target" || name.as_encoded_bytes().first() == Some(&b'.'))
    }
}

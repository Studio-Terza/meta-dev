mod failure;
mod golden_tests;
mod recorded_mutant;
mod shard_matrix;
mod summary;

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use golden_tests::GoldenTests;
use shard_matrix::ShardMatrix;
use summary::Summary;

use failure::Failure;

#[derive(Debug)]
pub struct Report {
    shards: PathBuf,
    sources: PathBuf,
    markdown: PathBuf,
    json: PathBuf,
}

impl Report {
    pub fn from_arguments(arguments: &[OsString]) -> Result<Report, Failure> {
        match arguments {
            [shards, sources, markdown, json] => Ok(Report {
                shards: PathBuf::from(shards),
                sources: PathBuf::from(sources),
                markdown: PathBuf::from(markdown),
                json: PathBuf::from(json),
            }),
            _ => Err(Failure::Usage),
        }
    }

    pub fn write_summary(&self) -> Result<(), Failure> {
        let matrices = self.matrices()?;
        let summary = Summary::from_results(
            &matrices.iter().flat_map(|matrix| matrix.baseline().iter().cloned()).collect(),
            &matrices.iter().flat_map(|matrix| matrix.mutants().iter().cloned()).collect::<Vec<_>>(),
            GoldenTests::under(&self.sources)?.names(),
        );
        File::create(&self.markdown)
            .and_then(|mut file| write!(file, "{summary}"))
            .map_err(|source| Failure::Io { path: self.markdown.clone(), source })?;
        let json = summary.json().map_err(|source| Failure::Parse { path: self.json.clone(), source })?;
        fs::write(&self.json, json).map_err(|source| Failure::Io { path: self.json.clone(), source })
    }

    fn matrices(&self) -> Result<Vec<ShardMatrix>, Failure> {
        let failed = |source| Failure::Io { path: self.shards.clone(), source };
        let mut directories: Vec<PathBuf> = fs::read_dir(&self.shards)
            .map_err(failed)?
            .map(|entry| entry.map(|found| found.path()))
            .collect::<Result<_, _>>()
            .map_err(failed)?;
        directories.sort();
        directories
            .iter()
            .map(|directory| Self::shard_file(directory))
            .filter(|file| file.is_file())
            .map(|file| ShardMatrix::read(&file))
            .collect()
    }

    fn shard_file(directory: &Path) -> PathBuf {
        directory.join(ShardMatrix::FILE)
    }
}

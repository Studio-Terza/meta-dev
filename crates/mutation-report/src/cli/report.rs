mod failure;
mod golden_tests;
mod recorded_mutant;
mod shard_matrix;
mod summary;
mod test_name;

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, Files, Kind};
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

    pub fn write_summary(&self, files: &impl Files) -> Result<(), Failure> {
        let matrices = self.matrices(files)?;
        let summary = Summary::from_results(&matrices, &GoldenTests::under(files, &self.sources)?);
        files
            .write(&self.markdown, &Bytes::from(&summary))
            .map_err(|failure| Failure::Io { path: self.markdown.clone(), source: io::Error::from(failure) })?;
        summary.written_as_json(files, &self.json)
    }

    fn matrices(&self, files: &impl Files) -> Result<Vec<ShardMatrix>, Failure> {
        let mut directories = files
            .entries(&self.shards)
            .map_err(|failure| Failure::Io { path: self.shards.clone(), source: io::Error::from(failure) })?;
        directories.sort();
        directories
            .iter()
            .map(|directory| Self::shard_file(directory))
            .filter(|file| files.kind(file) == Kind::File)
            .map(|file| ShardMatrix::read(files, &file))
            .collect()
    }

    fn shard_file(directory: &Path) -> PathBuf {
        directory.join(ShardMatrix::FILE)
    }
}

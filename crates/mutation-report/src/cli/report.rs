mod failure;
mod golden_tests;
mod recorded_mutant;
mod shard_matrix;
mod summary;
mod test_name;

use std::borrow::Cow;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, Files, Kind};
use golden_tests::GoldenTests;
use shard_matrix::ShardMatrix;
use summary::Summary;

pub use failure::Failure;

#[derive(Debug)]
pub struct Report<'arguments> {
    shards: &'arguments Path,
    sources: &'arguments Path,
    markdown: &'arguments Path,
    json: &'arguments Path,
}

impl<'arguments> Report<'arguments> {
    const ARGUMENTS: usize = 4;

    pub fn from_arguments(arguments: &'arguments [impl AsRef<OsStr>]) -> Result<Report<'arguments>, Failure<'static>> {
        TryInto::<&[_; Report::<'static>::ARGUMENTS]>::try_into(arguments)
            .map(|[shards, sources, markdown, json]| Report {
                shards: Path::new(shards),
                sources: Path::new(sources),
                markdown: Path::new(markdown),
                json: Path::new(json),
            })
            .map_err(Failure::Usage)
    }

    pub fn write_summary(&self, files: &impl Files) -> Result<(), Failure<'arguments>> {
        let matrices = self.matrices(files)?;
        let summary = Summary::from_results(&matrices, &GoldenTests::under(files, self.sources)?);
        files
            .write(self.markdown, Bytes::from(&summary))
            .map_err(|failure| Failure::Io { path: Cow::Borrowed(self.markdown), source: io::Error::from(failure) })?;
        summary.written_as_json(files, self.json)
    }

    fn matrices(&self, files: &impl Files) -> Result<Vec<ShardMatrix>, Failure<'arguments>> {
        let mut directories = files
            .entries(self.shards)
            .map_err(|failure| Failure::Io { path: Cow::Borrowed(self.shards), source: io::Error::from(failure) })?;
        directories.sort();
        let shard_files: Vec<PathBuf> = directories.iter().map(|directory| Self::shard_file(directory)).collect();
        let kinds = files.kinds(&shard_files);
        let present: Vec<PathBuf> =
            shard_files.into_iter().zip(kinds).filter(|(_, kind)| *kind == Kind::File).map(|(file, _)| file).collect();
        let read = files.read_text_many(&present);
        present.into_iter().zip(read).map(|(file, text)| ShardMatrix::parsed(file, text)).collect()
    }

    fn shard_file(directory: &Path) -> PathBuf {
        directory.join(ShardMatrix::FILE)
    }
}

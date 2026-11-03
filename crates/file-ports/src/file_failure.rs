use std::io;

#[derive(Debug, thiserror::Error)]
pub enum FileFailure {
    #[error("{0}")]
    Missing(io::Error),
    #[error("{0}")]
    Failed(io::Error),
}

impl From<io::Error> for FileFailure {
    fn from(source: io::Error) -> FileFailure {
        match source.kind() {
            io::ErrorKind::NotFound => FileFailure::Missing(source),
            _ => FileFailure::Failed(source),
        }
    }
}

impl From<FileFailure> for io::Error {
    fn from(failure: FileFailure) -> io::Error {
        match failure {
            FileFailure::Missing(source) | FileFailure::Failed(source) => source,
        }
    }
}

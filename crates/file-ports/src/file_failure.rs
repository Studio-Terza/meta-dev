use std::io;

#[derive(Debug, thiserror::Error)]
pub enum FileFailure {
    #[error("{0}")]
    Missing(io::Error),
    #[error("{0}")]
    Failed(io::Error),
}

impl From<io::Error> for FileFailure {
    fn from(source: io::Error) -> Self {
        match source.kind() {
            io::ErrorKind::NotFound => Self::Missing(source),
            _ => Self::Failed(source),
        }
    }
}

impl From<FileFailure> for io::Error {
    fn from(failure: FileFailure) -> Self {
        match failure {
            FileFailure::Missing(source) | FileFailure::Failed(source) => source,
        }
    }
}

use std::io;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct FileFailure(#[from] io::Error);

impl From<FileFailure> for io::Error {
    fn from(failure: FileFailure) -> io::Error {
        failure.0
    }
}

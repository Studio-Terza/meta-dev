use std::io;

#[derive(Debug, thiserror::Error)]
pub enum DispatchFailure {
    #[error("cannot start the program: {0}")]
    Spawn(#[source] io::Error),
    #[error("cannot read the program's output: {0}")]
    Collect(#[source] io::Error),
    #[error("a thread exchanging input and output with the program ended abnormally")]
    Unjoined,
    #[error("cannot kill a process group: {0}")]
    Kill(#[source] io::Error),
}

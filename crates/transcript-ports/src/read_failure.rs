use file_ports::FileFailure;

#[derive(Debug, thiserror::Error)]
pub enum ReadFailure {
    #[error("cannot open a transcript: {0}")]
    File(#[from] FileFailure),
}

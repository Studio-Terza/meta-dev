use file_ports::FileFailure;

#[derive(Debug, thiserror::Error)]
pub enum ReadFailure {
    #[error("cannot open a transcript: {0}")]
    File(#[from] FileFailure),
    #[error("a transcript line is not a record: {0}")]
    Malformed(#[from] serde_json::Error),
}

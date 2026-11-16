use file_ports::{FileFailure, Offset};

#[derive(Debug, thiserror::Error)]
pub enum ReadFailure {
    #[error("cannot open a transcript: {0}")]
    File(#[from] FileFailure),
    #[error("a transcript read past {0} runs beyond the largest offset a file has")]
    Uncounted(Offset),
}

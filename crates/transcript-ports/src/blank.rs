#[derive(Debug, thiserror::Error)]
#[error("an identifier or timestamp is blank")]
pub struct Blank;

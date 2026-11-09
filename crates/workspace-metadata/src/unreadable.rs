#[derive(Debug, thiserror::Error)]
#[error("cargo metadata is not readable: {0}")]
pub struct Unreadable(#[from] serde_json::Error);

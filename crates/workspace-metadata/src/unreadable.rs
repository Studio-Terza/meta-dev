#[derive(Debug, thiserror::Error)]
#[error("cannot parse cargo metadata: {0}")]
pub struct Unreadable(#[from] serde_json::Error);

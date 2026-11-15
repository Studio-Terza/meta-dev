use std::fmt;
use std::path::PathBuf;

use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct CacheFile(String);

#[derive(Debug, thiserror::Error)]
pub enum Unexpanded {
    #[error("no {0} in any .sqlx above this crate; regenerate the query cache")]
    Uncached(CacheFile),
    #[error("cannot read the cached query {}: {source}", path.display())]
    Unread { path: PathBuf, source: std::io::Error },
    #[error("cannot parse the cached query {}: {source}", path.display())]
    Unparsed { path: PathBuf, source: serde_json::Error },
    #[error("the query takes {wanted} parameters but {given} are bound")]
    Unbound { wanted: usize, given: usize },
}

impl From<&str> for CacheFile {
    fn from(sql: &str) -> CacheFile {
        CacheFile(hex::encode(Sha256::digest(sql.as_bytes())))
    }
}

impl fmt::Display for CacheFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "query-{}.json", self.0)
    }
}

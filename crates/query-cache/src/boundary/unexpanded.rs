use std::fmt;
use std::path::PathBuf;

use file_ports::FileFailure;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CacheFile(String);

#[derive(Debug, thiserror::Error)]
pub enum Unexpanded {
    #[error(
        "no {0} in any .sqlx above this crate; regenerate the query cache with cargo run --bin ci -- queries . --record"
    )]
    Uncached(CacheFile),
    #[error("cannot read the cached query {}: {source}", path.display())]
    Unread { path: PathBuf, source: FileFailure },
    #[error("cannot parse the cached query {}: {source}", path.display())]
    Unparsed { path: PathBuf, source: serde_json::Error },
    #[error("the query takes {wanted} parameters but {given} are bound")]
    Unbound { wanted: usize, given: usize },
}

impl From<&str> for CacheFile {
    fn from(sql: &str) -> CacheFile {
        CacheFile(format!("query-{}.json", hex::encode(Sha256::digest(sql.as_bytes()))))
    }
}

impl From<String> for CacheFile {
    fn from(named: String) -> CacheFile {
        CacheFile(named)
    }
}

impl AsRef<str> for CacheFile {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CacheFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

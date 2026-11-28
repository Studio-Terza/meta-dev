use std::fmt;
use std::path::PathBuf;

use digest_ports::{Digested, Hex};
use json_ports::JsonFailure;

#[derive(Debug)]
pub struct CacheFile(String);

#[derive(Debug, thiserror::Error)]
pub enum Unexpanded {
    #[error(
        "no {0} in any .sqlx above this crate; regenerate the query cache with FIXTURE_RECORD=1 cargo test -p syntax"
    )]
    Uncached(CacheFile),
    #[error("cannot read the cached query {}: {source}", path.display())]
    Unread { path: PathBuf, source: std::io::Error },
    #[error("cannot parse the cached query {}: {source}", path.display())]
    Unparsed { path: PathBuf, source: JsonFailure },
    #[error("the query takes {wanted} parameters but {given} are bound")]
    Unbound { wanted: usize, given: usize },
}

impl From<&str> for CacheFile {
    fn from(sql: &str) -> CacheFile {
        CacheFile(Hex::encoded(Digested::of(sql.as_bytes()).bytes()))
    }
}

impl fmt::Display for CacheFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "query-{}.json", self.0)
    }
}

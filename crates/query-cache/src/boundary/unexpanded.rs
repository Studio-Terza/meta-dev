use std::fmt;
use std::path::PathBuf;

use proc_macro2::Ident;
use sha2::{Digest, Sha256};
use syn::LitStr;

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
    #[error("column {0} is described with a type that maps to no Rust type; name its type, as in \"{0}!: String\"")]
    Unmapped(Ident),
}

impl From<&LitStr> for CacheFile {
    fn from(source: &LitStr) -> CacheFile {
        CacheFile(hex::encode(Sha256::digest(source.value().as_bytes())))
    }
}

impl fmt::Display for CacheFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "query-{}.json", self.0)
    }
}

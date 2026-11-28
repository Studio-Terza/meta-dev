use std::fmt::Display;

use port_core::port;
use query_cache::Entry;

#[port]
pub trait Describe {
    fn describe(&self, query: &str) -> Result<Entry<String>, Undescribed>;
}

#[derive(Debug, thiserror::Error)]
#[error("cannot describe {query:?}: {reason}")]
pub struct Undescribed {
    query: String,
    reason: String,
}

impl Undescribed {
    #[must_use]
    pub fn told(query: &str, reason: &impl Display) -> Undescribed {
        Undescribed { query: query.to_owned(), reason: reason.to_string() }
    }
}

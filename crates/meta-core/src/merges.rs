use crate::ids::PrNumber;
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Merges(Vec<PrNumber>);

/// The GitHub merged-PR list as `meta status` can use it: the rows, or why they cannot be trusted.
#[derive(Debug)]
pub enum Reconciliation {
    Listed(Merges),
    Unavailable(Cause),
}

#[derive(Debug)]
pub enum Cause {
    Port(Unreached),
    Unparsed(serde_json::Error),
    Truncated,
}

/// What a port reported when it could not produce the list. Only the report reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unreached(String);

impl Unreached {
    pub fn by(failure: &impl fmt::Display) -> Self {
        Self(failure.to_string())
    }
}

#[derive(Deserialize)]
struct Listed {
    number: PrNumber,
}

impl Merges {
    /// The most rows the adapter may keep; it asks for one more so that a full page shows as truncation.
    pub const LIMIT: usize = 1000;

    pub fn from_gh(listed: &str) -> Reconciliation {
        match serde_json::from_str::<Vec<Listed>>(listed) {
            Err(error) => Reconciliation::Unavailable(Cause::Unparsed(error)),
            Ok(rows) if rows.len() > Self::LIMIT => Reconciliation::Unavailable(Cause::Truncated),
            Ok(rows) => Reconciliation::Listed(Self(rows.into_iter().map(|row| row.number).collect())),
        }
    }

    pub(crate) fn prs(&self) -> &[PrNumber] {
        &self.0
    }
}

impl fmt::Display for Cause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Port(Unreached(why)) => write!(formatter, "{why}"),
            Self::Unparsed(error) => write!(formatter, "gh pr list merged: {error}"),
            Self::Truncated => write!(formatter, "the list is truncated: {} or more rows", Merges::LIMIT + 1),
        }
    }
}

use crate::ids::{Malformed, PrNumber};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Merges(Vec<PrNumber>);

#[derive(Deserialize)]
struct Listed {
    number: PrNumber,
}

impl Merges {
    pub fn from_gh(listed: &str) -> Result<Self, Malformed> {
        let rows: Vec<Listed> =
            serde_json::from_str(listed).map_err(|error| Malformed::because(format!("gh pr list merged: {error}")))?;
        Ok(Self(rows.into_iter().map(|row| row.number).collect()))
    }

    pub(crate) fn prs(&self) -> &[PrNumber] {
        &self.0
    }
}

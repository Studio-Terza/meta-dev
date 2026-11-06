use serde::Deserialize;

use crate::Kind;

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Unrecognised {
    #[serde(rename = "type", alias = "kind")]
    kind: Kind,
}

impl Unrecognised {
    #[must_use]
    pub fn kind(&self) -> &Kind {
        &self.kind
    }
}

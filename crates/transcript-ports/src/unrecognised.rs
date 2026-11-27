mod boundary;

use serde::Deserialize;

use crate::{Kind, Uuid};

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Unrecognised {
    #[serde(rename = "type", alias = "kind")]
    kind: Kind,
    uuid: Option<Uuid>,
    #[serde(rename = "parentUuid")]
    parent: Option<Uuid>,
}

impl Unrecognised {
    const UNREADABLE: &'static str = "unreadable";

    #[must_use]
    pub fn kind(&self) -> &Kind {
        &self.kind
    }

    pub(crate) fn within(self, uuid: Uuid, parent: Option<Uuid>) -> Unrecognised {
        Unrecognised { kind: self.kind, uuid: Some(uuid), parent }
    }
}

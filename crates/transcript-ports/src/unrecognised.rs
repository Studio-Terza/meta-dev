use serde::Deserialize;

use crate::{Kind, Skipped, Uuid};

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

    #[must_use]
    pub fn passing(&self) -> Option<Skipped> {
        self.uuid.clone().map(|uuid| Skipped::new(uuid, self.parent.clone()))
    }

    pub(crate) fn unreadable() -> Option<Unrecognised> {
        Kind::try_new(Self::UNREADABLE.to_owned()).ok().map(|kind| Unrecognised { kind, uuid: None, parent: None })
    }

    pub(crate) fn within(self, uuid: Uuid, parent: Option<Uuid>) -> Unrecognised {
        Unrecognised { kind: self.kind, uuid: Some(uuid), parent }
    }
}

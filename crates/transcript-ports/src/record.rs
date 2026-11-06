mod boundary;

use crate::{Authorship, Body, Instant, SessionId, Uuid};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    uuid: Uuid,
    parent: Option<Uuid>,
    session: SessionId,
    at: Instant,
    author: Authorship,
    text: Body,
}

impl Record {
    #[must_use]
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    #[must_use]
    pub fn parent(&self) -> Option<&Uuid> {
        self.parent.as_ref()
    }

    #[must_use]
    pub fn session(&self) -> &SessionId {
        &self.session
    }

    #[must_use]
    pub fn stamp(&self) -> &Instant {
        &self.at
    }

    #[must_use]
    pub fn author(&self) -> Authorship {
        self.author
    }

    #[must_use]
    pub fn text(&self) -> &Body {
        &self.text
    }
}

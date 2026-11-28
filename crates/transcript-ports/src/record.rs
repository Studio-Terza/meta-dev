mod boundary;

use crate::{Authorship, Body, Instant, SessionId, Skipped, Uuid};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    uuid: Uuid,
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

impl Skipped {
    #[must_use]
    pub fn said(&self, session: SessionId, at: Instant, author: Authorship, text: Body) -> Record {
        Record { uuid: self.uuid().clone(), session, at, author, text }
    }
}

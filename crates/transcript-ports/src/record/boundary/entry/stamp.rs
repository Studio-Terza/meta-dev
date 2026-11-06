use serde::Deserialize;

use crate::{Authorship, Body, Instant, Line, Record, SessionId, Skipped, Uuid};

#[derive(Clone, Copy, Debug)]
struct Side;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(from = "bool")]
struct Chain(Option<Side>);

#[derive(Debug, Deserialize)]
pub struct Stamp {
    uuid: Uuid,
    #[serde(rename = "parentUuid")]
    parent: Option<Uuid>,
    #[serde(rename = "sessionId")]
    session: SessionId,
    timestamp: Instant,
    #[serde(rename = "isSidechain", default)]
    chain: Chain,
}

#[derive(Debug)]
struct Main(Stamp);

#[derive(Debug, Deserialize)]
pub struct Passing {
    uuid: Option<Uuid>,
    #[serde(rename = "parentUuid")]
    parent: Option<Uuid>,
}

impl From<bool> for Chain {
    fn from(side: bool) -> Chain {
        Chain(side.then_some(Side))
    }
}

impl From<Stamp> for Skipped {
    fn from(stamp: Stamp) -> Skipped {
        Skipped::new(stamp.uuid, stamp.parent)
    }
}

impl Stamp {
    pub fn said(self, author: Authorship, text: Body) -> Line {
        match self.chain.0 {
            None => Line::Said(Main(self).voiced(author, text)),
            Some(Side) => Line::Skipped(Skipped::from(self)),
        }
    }
}

impl Main {
    fn voiced(self, author: Authorship, text: Body) -> Record {
        let Main(Stamp { uuid, parent, session, timestamp, .. }) = self;
        Record { uuid, parent, session, at: timestamp, author, text }
    }
}

impl Passing {
    pub fn line(self) -> Option<Line> {
        self.uuid.map(|uuid| Line::Skipped(Skipped::new(uuid, self.parent)))
    }
}

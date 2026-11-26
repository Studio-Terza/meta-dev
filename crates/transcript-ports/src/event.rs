use std::fmt;
use std::str::FromStr;

use newtype::Spelled;

use crate::{Body, Instant, SessionId, Skipped, Uuid};

#[derive(Spelled, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Happening {
    Notice,
    Relayed,
    Launch,
    Sent,
}

#[derive(Debug, thiserror::Error)]
#[error("{0:?} names no notice, relayed message or launch")]
pub struct Unhappened(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    uuid: Uuid,
    parent: Option<Uuid>,
    session: SessionId,
    at: Instant,
    happening: Happening,
    text: Body,
}

#[derive(Debug)]
pub(crate) struct Placed {
    uuid: Uuid,
    parent: Option<Uuid>,
    session: SessionId,
    at: Instant,
}

impl Happening {
    const ALL: [Happening; 4] = [Happening::Notice, Happening::Relayed, Happening::Launch, Happening::Sent];
}

impl fmt::Display for Happening {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Happening::Notice => formatter.write_str("notice"),
            Happening::Relayed => formatter.write_str("relayed"),
            Happening::Launch => formatter.write_str("launch"),
            Happening::Sent => formatter.write_str("sent"),
        }
    }
}

impl FromStr for Happening {
    type Err = Unhappened;

    fn from_str(text: &str) -> Result<Happening, Unhappened> {
        Self::ALL.into_iter().find(|happening| happening.to_string() == text).ok_or_else(|| Unhappened(text.to_owned()))
    }
}

impl Placed {
    pub(crate) fn new(uuid: Uuid, parent: Option<Uuid>, session: SessionId, at: Instant) -> Placed {
        Placed { uuid, parent, session, at }
    }

    pub(crate) fn happened(self, happening: Happening, text: Body) -> Event {
        let Placed { uuid, parent, session, at } = self;
        Event { uuid, parent, session, at, happening, text }
    }
}

impl Event {
    #[must_use]
    pub fn happening(&self) -> Happening {
        self.happening
    }
}

impl AsRef<Uuid> for Event {
    fn as_ref(&self) -> &Uuid {
        &self.uuid
    }
}

impl AsRef<SessionId> for Event {
    fn as_ref(&self) -> &SessionId {
        &self.session
    }
}

impl AsRef<Instant> for Event {
    fn as_ref(&self) -> &Instant {
        &self.at
    }
}

impl AsRef<Body> for Event {
    fn as_ref(&self) -> &Body {
        &self.text
    }
}

impl From<&Event> for Skipped {
    fn from(event: &Event) -> Skipped {
        Skipped::new(event.uuid.clone(), event.parent.clone())
    }
}

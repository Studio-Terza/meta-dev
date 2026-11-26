use serde::Deserialize;

use super::said::Hears;
use super::stamp::Stamp;
use crate::{Authorship, Body, Line, Skipped, Unrecognised};

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Source {
    Human,
    TaskNotification,
    Unclassified,
    Coordinator,
    Peer,
    AutoContinuation,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Origin {
    Known(Source),
    Unknown(Unrecognised),
}

#[derive(Debug)]
pub struct Listener {
    stamp: Stamp,
    origin: Option<Origin>,
}

impl Source {
    fn spoke(self, stamp: Stamp, text: Body) -> Line {
        match self {
            Source::Human => stamp.said(Authorship::Owner, text),
            Source::TaskNotification
            | Source::Unclassified
            | Source::Coordinator
            | Source::Peer
            | Source::AutoContinuation => Line::Skipped(Skipped::from(stamp)),
        }
    }
}

impl Origin {
    fn spoke(self, stamp: Stamp, text: Body) -> Line {
        match self {
            Origin::Known(source) => source.spoke(stamp, text),
            Origin::Unknown(unrecognised) => stamp.unrecognised(unrecognised),
        }
    }

    fn silent(self, stamp: Stamp) -> Line {
        match self {
            Origin::Known(_) => Line::Skipped(Skipped::from(stamp)),
            Origin::Unknown(unrecognised) => stamp.unrecognised(unrecognised),
        }
    }
}

impl Listener {
    pub fn new(stamp: Stamp, origin: Option<Origin>) -> Listener {
        Listener { stamp, origin }
    }
}

impl Hears for Listener {
    fn text(self, text: Body) -> Line {
        match self.origin {
            Some(origin) => origin.spoke(self.stamp, text),
            None => Line::Skipped(Skipped::from(self.stamp)),
        }
    }

    fn silence(self) -> Line {
        match self.origin {
            Some(origin) => origin.silent(self.stamp),
            None => Line::Skipped(Skipped::from(self.stamp)),
        }
    }

    fn unrecognised(self, unrecognised: Unrecognised) -> Line {
        self.stamp.unrecognised(unrecognised)
    }
}

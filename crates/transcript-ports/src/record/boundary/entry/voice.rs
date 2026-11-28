use grammar::Grammar;
use grammar::notice::TaskNotice;
use invariants::Ignoring;

use super::object::Object;
use super::said::Hears;
use super::stamp::Stamp;
use crate::{Authorship, Body, Ignored, Line, Skipped, TaskId, Unrecognised};

#[derive(strum::EnumString, Clone, Copy, Debug)]
#[strum(serialize_all = "kebab-case")]
pub enum Source {
    Human,
    TaskNotification,
    Unclassified,
    Coordinator,
    Peer,
    AutoContinuation,
}

#[derive(Debug)]
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
            Source::TaskNotification => TaskNotice::read(text.as_ref())
                .and_then(|notice| TaskId::try_new(notice.task().to_owned()).ignoring(Ignored::Unstopped))
                .map_or_else(|| Line::Skipped(Skipped::from(stamp)), Line::Ended),
            Source::Unclassified | Source::Coordinator | Source::Peer | Source::AutoContinuation => {
                Line::Skipped(Skipped::from(stamp))
            }
        }
    }
}

impl Origin {
    const KIND: &'static str = "kind";

    pub fn read(origin: Object<'_>) -> Result<Origin, serde_json::Error> {
        let source = origin.tag(Self::KIND).and_then(|kind| kind.parse::<Source>().ignoring(Ignored::NotKnown));
        source.map_or_else(|| origin.whole().map(Origin::Unknown), |known| Ok(Origin::Known(known)))
    }

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

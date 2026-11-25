mod links;

use std::collections::BTreeSet;
use std::ops::Range;

use algebra::Semigroup;
use newtype::proptest::arbitrary::any;
use newtype::proptest::collection::vec;
use newtype::proptest::strategy::Strategy;
use newtype::{Arbitrary, Display, Newtype, Wrapping};
use transcript_ports::{Line, Record, Skipped, Unrecognised, Uuid};

use crate::{Earlier, MechanicalLink, Message};
use links::Links;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Earliest(usize);

#[derive(Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Resumed {
    messages: Earliest,
    skipped: Earliest,
}

#[derive(Arbitrary, Clone, Debug, Default, PartialEq, Eq)]
#[newtype(generate = Stream::sampled)]
pub struct Stream {
    messages: Vec<Message>,
    skipped: Vec<Skipped>,
    unrecognised: Vec<Unrecognised>,
    resumed: Resumed,
}

impl FromIterator<Line> for Stream {
    fn from_iter<Lines: IntoIterator<Item = Line>>(lines: Lines) -> Stream {
        Earlier::default().resumed(lines)
    }
}

impl Semigroup for Stream {
    fn combine(self, later: Stream) -> Stream {
        Stream {
            messages: self.messages.combine(later.messages),
            skipped: self.skipped.combine(later.skipped),
            unrecognised: self.unrecognised.combine(later.unrecognised),
            resumed: self.resumed,
        }
    }
}

impl From<Record> for Stream {
    fn from(record: Record) -> Stream {
        Stream { messages: vec![Message::from(record)], ..Stream::default() }
    }
}

impl From<Skipped> for Stream {
    fn from(skipped: Skipped) -> Stream {
        Stream { skipped: vec![skipped], ..Stream::default() }
    }
}

impl From<Unrecognised> for Stream {
    fn from(unrecognised: Unrecognised) -> Stream {
        Stream {
            skipped: unrecognised.passing().into_iter().collect(),
            unrecognised: vec![unrecognised],
            ..Stream::default()
        }
    }
}

impl AsRef<[Message]> for Stream {
    fn as_ref(&self) -> &[Message] {
        self.messages.get(self.resumed.messages.into_inner()..).unwrap_or_default()
    }
}

impl Stream {
    const SAMPLED: Range<usize> = 0..4;

    fn sampled() -> impl Strategy<Value = Stream> {
        Strategy::prop_map(
            (
                vec(any::<Message>(), Self::SAMPLED),
                vec(any::<Skipped>(), Self::SAMPLED),
                vec(any::<Unrecognised>(), Self::SAMPLED),
                any::<Resumed>(),
            ),
            |(messages, skipped, unrecognised, resumed)| Stream { messages, skipped, unrecognised, resumed },
        )
    }

    pub(crate) fn after(messages: Vec<Message>, skipped: Vec<Skipped>) -> Stream {
        let resumed = Resumed { messages: Earliest::new(messages.len()), skipped: Earliest::new(skipped.len()) };
        Stream { messages, skipped, unrecognised: Vec::new(), resumed }
    }

    #[must_use]
    pub fn links(&self) -> Vec<MechanicalLink> {
        let fresh: BTreeSet<&Uuid> = self.as_ref().iter().map(|message| message.record().uuid()).collect();
        Links::over(&self.messages, &self.skipped)
            .all()
            .into_iter()
            .filter(|link| link.anchors().iter().any(|anchor| fresh.contains(anchor.message())))
            .collect()
    }

    #[must_use]
    pub fn skipped(&self) -> &[Skipped] {
        self.skipped.get(self.resumed.skipped.into_inner()..).unwrap_or_default()
    }

    #[must_use]
    pub fn unrecognised(&self) -> &[Unrecognised] {
        &self.unrecognised
    }
}

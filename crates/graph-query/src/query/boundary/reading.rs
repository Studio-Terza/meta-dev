use std::ffi::OsString;
use std::path::PathBuf;

use lint_layout::Name;
use lint_transition_graph::Ask;

use algebra::Semigroup;
use strum::{IntoStaticStr, VariantArray};

use crate::boundary::QueryFailure;
use crate::query::top::Top;
use crate::query::{Consumer, Query, Term};
use crate::role::Role;

#[derive(IntoStaticStr, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case", prefix = "--")]
enum Item {
    Defined,
    Uses,
    Users,
    Implementers,
    Cluster,
    Exports,
    Cycles,
}

#[derive(IntoStaticStr, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case", prefix = "--")]
enum Switch {
    In,
    Role,
    Top,
    Path,
    Crates,
    ConsumedBy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Expecting {
    Word,
    Root,
    Role,
    Top,
    Item(Item),
    From,
    To(Name),
    Crates,
    Consumer,
}

#[derive(Debug)]
pub(crate) struct Reading {
    query: Query,
    expecting: Expecting,
}

impl Query {
    pub fn read(arguments: &[OsString]) -> Result<Query, QueryFailure> {
        let start = Reading { query: Query::default(), expecting: Expecting::Word };
        let finished = arguments.iter().try_fold(start, |reading, argument| {
            argument.to_str().ok_or(QueryFailure::Unreadable).and_then(|word| reading.next(word))
        })?;
        finished.done()
    }
}

impl Expecting {
    fn flag(word: &str) -> Option<Expecting> {
        Switch::VARIANTS
            .iter()
            .find(|switch| <&str>::from(**switch) == word)
            .map(|switch| switch.expecting())
            .or_else(|| Item::VARIANTS.iter().copied().find(|item| <&str>::from(*item) == word).map(Expecting::Item))
    }
}

impl Switch {
    fn expecting(self) -> Expecting {
        match self {
            Switch::In => Expecting::Root,
            Switch::Role => Expecting::Role,
            Switch::Top => Expecting::Top,
            Switch::Path => Expecting::From,
            Switch::Crates => Expecting::Crates,
            Switch::ConsumedBy => Expecting::Consumer,
        }
    }
}

impl Item {
    fn ask(self, name: Name) -> Ask {
        match self {
            Item::Defined => Ask::Defined(name),
            Item::Uses => Ask::Uses(name),
            Item::Users => Ask::Users(name),
            Item::Implementers => Ask::Implementers(name),
            Item::Cluster => Ask::Cluster(name),
            Item::Exports => Ask::Exports(name),
            Item::Cycles => Ask::Cycles(name),
        }
    }
}

impl From<Query> for Reading {
    fn from(query: Query) -> Reading {
        Reading { query, expecting: Expecting::Word }
    }
}

impl Reading {
    fn next(self, word: &str) -> Result<Reading, QueryFailure> {
        match self.expecting {
            Expecting::Word | Expecting::Crates => self.flagged(Expecting::flag(word), word),
            Expecting::Root => Ok(Reading::from(self.query.combine(Query::from(PathBuf::from(word))))),
            Expecting::Consumer => {
                Ok(Reading::from(self.query.combine(Query::from(Consumer::new(PathBuf::from(word))))))
            }
            Expecting::Role => Ok(Reading::from(self.query.combine(Query::from(word.parse::<Role>()?)))),
            Expecting::Top => Ok(Reading::from(self.query.combine(Query::from(word.parse::<Top>()?)))),
            Expecting::Item(item) => Ok(Reading::from(self.query.combine(Query::from(item.ask(Name::from(word)))))),
            Expecting::From => Ok(Reading { query: self.query, expecting: Expecting::To(Name::from(word)) }),
            Expecting::To(from) => {
                Ok(Reading::from(self.query.combine(Query::from(Ask::Path(from, Name::from(word))))))
            }
        }
    }

    fn flagged(self, flag: Option<Expecting>, word: &str) -> Result<Reading, QueryFailure> {
        match flag {
            Some(Expecting::Crates) => Ok(Reading::from(self.query.combine(Query::CRATES))),
            Some(expecting) => Ok(Reading { query: self.query, expecting }),
            None => Ok(Reading::from(self.query.combine(Query::from(word.parse::<Term>()?)))),
        }
    }

    fn done(self) -> Result<Query, QueryFailure> {
        match self.expecting {
            Expecting::Word => Self::asked(self.query),
            Expecting::Root
            | Expecting::Role
            | Expecting::Top
            | Expecting::Item(_)
            | Expecting::From
            | Expecting::To(_)
            | Expecting::Crates
            | Expecting::Consumer => Err(QueryFailure::Unfinished),
        }
    }

    fn asked(query: Query) -> Result<Query, QueryFailure> {
        query.asks().map(|_| ()).ok_or(QueryFailure::Empty).map(|()| query)
    }
}

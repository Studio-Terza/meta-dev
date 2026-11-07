use std::ffi::OsString;
use std::path::PathBuf;

use lint_layout::Name;
use lint_transition_graph::Ask;

use crate::boundary::QueryFailure;
use crate::query::Query;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Item {
    Defined,
    Uses,
    Users,
    Implementers,
    Cluster,
    Exports,
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
}

#[derive(Debug)]
pub(crate) struct Reading {
    query: Query,
    expecting: Expecting,
}

impl TryFrom<&[OsString]> for Query {
    type Error = QueryFailure;

    fn try_from(arguments: &[OsString]) -> Result<Self, QueryFailure> {
        let start = Reading { query: Self::default(), expecting: Expecting::Word };
        let finished = arguments.iter().try_fold(start, |reading, argument| {
            argument.to_str().ok_or(QueryFailure::Unreadable).and_then(|word| reading.next(word))
        })?;
        finished.done()
    }
}

impl Expecting {
    const FLAGS: [(&'static str, Self); 10] = [
        ("--in", Self::Root),
        ("--role", Self::Role),
        ("--top", Self::Top),
        ("--defined", Self::Item(Item::Defined)),
        ("--uses", Self::Item(Item::Uses)),
        ("--users", Self::Item(Item::Users)),
        ("--implementers", Self::Item(Item::Implementers)),
        ("--cluster", Self::Item(Item::Cluster)),
        ("--exports", Self::Item(Item::Exports)),
        ("--path", Self::From),
    ];

    fn flag(word: &str) -> Option<Self> {
        Self::FLAGS.into_iter().find(|(flag, _)| *flag == word).map(|(_, expecting)| expecting)
    }
}

impl Item {
    const fn ask(self, name: Name) -> Ask {
        match self {
            Self::Defined => Ask::Defined(name),
            Self::Uses => Ask::Uses(name),
            Self::Users => Ask::Users(name),
            Self::Implementers => Ask::Implementers(name),
            Self::Cluster => Ask::Cluster(name),
            Self::Exports => Ask::Exports(name),
        }
    }
}

impl Reading {
    fn next(self, word: &str) -> Result<Self, QueryFailure> {
        match self.expecting {
            Expecting::Word => self.flagged(Expecting::flag(word), word),
            Expecting::Root => {
                Ok(Self { query: self.query.with_root(PathBuf::from(word)), expecting: Expecting::Word })
            }
            Expecting::Role => Ok(Self { query: self.query.with_role(word.parse()?), expecting: Expecting::Word }),
            Expecting::Top => Ok(Self { query: self.query.with_top(word.parse()?), expecting: Expecting::Word }),
            Expecting::Item(item) => {
                Ok(Self { query: self.query.with_ask(item.ask(Name::from(word))), expecting: Expecting::Word })
            }
            Expecting::From => Ok(Self { query: self.query, expecting: Expecting::To(Name::from(word)) }),
            Expecting::To(from) => {
                Ok(Self { query: self.query.with_ask(Ask::Path(from, Name::from(word))), expecting: Expecting::Word })
            }
        }
    }

    fn flagged(self, flag: Option<Expecting>, word: &str) -> Result<Self, QueryFailure> {
        match flag {
            Some(expecting) => Ok(Self { query: self.query, expecting }),
            None => Ok(Self { query: self.query.with_term(word.parse()?), expecting: Expecting::Word }),
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
            | Expecting::To(_) => Err(QueryFailure::Unfinished),
        }
    }

    fn asked(query: Query) -> Result<Query, QueryFailure> {
        query.asks().map(|_| ()).ok_or(QueryFailure::Empty).map(|()| query)
    }
}

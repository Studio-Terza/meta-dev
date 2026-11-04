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

    fn try_from(arguments: &[OsString]) -> Result<Query, QueryFailure> {
        let start = Reading { query: Query::default(), expecting: Expecting::Word };
        let finished = arguments.iter().try_fold(start, |reading, argument| {
            argument.to_str().ok_or(QueryFailure::Unreadable).and_then(|word| reading.next(word))
        })?;
        finished.done()
    }
}

impl Expecting {
    const FLAGS: [(&'static str, Expecting); 10] = [
        ("--in", Expecting::Root),
        ("--role", Expecting::Role),
        ("--top", Expecting::Top),
        ("--defined", Expecting::Item(Item::Defined)),
        ("--uses", Expecting::Item(Item::Uses)),
        ("--users", Expecting::Item(Item::Users)),
        ("--implementers", Expecting::Item(Item::Implementers)),
        ("--cluster", Expecting::Item(Item::Cluster)),
        ("--exports", Expecting::Item(Item::Exports)),
        ("--path", Expecting::From),
    ];

    fn flag(word: &str) -> Option<Expecting> {
        Self::FLAGS.into_iter().find(|(flag, _)| *flag == word).map(|(_, expecting)| expecting)
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
        }
    }
}

impl Reading {
    fn next(self, word: &str) -> Result<Reading, QueryFailure> {
        match self.expecting {
            Expecting::Word => self.flagged(Expecting::flag(word), word),
            Expecting::Root => {
                Ok(Reading { query: self.query.with_root(PathBuf::from(word)), expecting: Expecting::Word })
            }
            Expecting::Role => Ok(Reading { query: self.query.with_role(word.parse()?), expecting: Expecting::Word }),
            Expecting::Top => Ok(Reading { query: self.query.with_top(word.parse()?), expecting: Expecting::Word }),
            Expecting::Item(item) => {
                Ok(Reading { query: self.query.with_ask(item.ask(Name::from(word))), expecting: Expecting::Word })
            }
            Expecting::From => Ok(Reading { query: self.query, expecting: Expecting::To(Name::from(word)) }),
            Expecting::To(from) => Ok(Reading {
                query: self.query.with_ask(Ask::Path(from, Name::from(word))),
                expecting: Expecting::Word,
            }),
        }
    }

    fn flagged(self, flag: Option<Expecting>, word: &str) -> Result<Reading, QueryFailure> {
        match flag {
            Some(expecting) => Ok(Reading { query: self.query, expecting }),
            None => Ok(Reading { query: self.query.with_term(word.parse()?), expecting: Expecting::Word }),
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

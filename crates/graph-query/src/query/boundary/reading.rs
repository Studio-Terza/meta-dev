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

#[derive(Clone, Debug, PartialEq, Eq)]
enum Spelling {
    Flag(Expecting),
    Unknown,
    Term,
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
    const FLAG_MARK: char = '-';

    fn flag(word: &str) -> Option<Expecting> {
        Flag::ALL.into_iter().find(|flag| flag.spelling == word).map(|flag| flag.expecting)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Arity {
    One,
    Two,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Flag {
    spelling: &'static str,
    arity: Arity,
    expecting: Expecting,
}

impl Flag {
    pub(crate) const ALL: [Flag; 10] = [
        Flag { spelling: "--in", arity: Arity::One, expecting: Expecting::Root },
        Flag { spelling: "--role", arity: Arity::One, expecting: Expecting::Role },
        Flag { spelling: "--top", arity: Arity::One, expecting: Expecting::Top },
        Flag { spelling: "--defined", arity: Arity::One, expecting: Expecting::Item(Item::Defined) },
        Flag { spelling: "--uses", arity: Arity::One, expecting: Expecting::Item(Item::Uses) },
        Flag { spelling: "--users", arity: Arity::One, expecting: Expecting::Item(Item::Users) },
        Flag { spelling: "--implementers", arity: Arity::One, expecting: Expecting::Item(Item::Implementers) },
        Flag { spelling: "--cluster", arity: Arity::One, expecting: Expecting::Item(Item::Cluster) },
        Flag { spelling: "--exports", arity: Arity::One, expecting: Expecting::Item(Item::Exports) },
        Flag { spelling: "--path", arity: Arity::Two, expecting: Expecting::From },
    ];

    pub(crate) fn spelling(&self) -> &'static str {
        self.spelling
    }

    pub(crate) fn arity(&self) -> Arity {
        self.arity
    }
}

impl From<&str> for Spelling {
    fn from(word: &str) -> Spelling {
        match Expecting::flag(word) {
            Some(expecting) => Spelling::Flag(expecting),
            None => Self::unflagged(word.chars().next()),
        }
    }
}

impl Spelling {
    fn unflagged(first: Option<char>) -> Spelling {
        match first.filter(|first| *first == Expecting::FLAG_MARK) {
            Some(_) => Spelling::Unknown,
            None => Spelling::Term,
        }
    }
}

impl Spelling {
    fn name(word: &str) -> Result<Name, QueryFailure> {
        match Spelling::from(word) {
            Spelling::Flag(_) => Err(QueryFailure::Unfinished),
            Spelling::Unknown => Err(QueryFailure::Flag(word.to_owned())),
            Spelling::Term => Ok(Name::from(word)),
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
        }
    }
}

impl Reading {
    fn next(self, word: &str) -> Result<Reading, QueryFailure> {
        match self.expecting {
            Expecting::Word => self.flagged(Spelling::from(word), word),
            Expecting::Root => {
                Ok(Reading { query: self.query.with_root(PathBuf::from(word)), expecting: Expecting::Word })
            }
            Expecting::Role => Ok(Reading { query: self.query.with_role(word.parse()?), expecting: Expecting::Word }),
            Expecting::Top => Ok(Reading { query: self.query.with_top(word.parse()?), expecting: Expecting::Word }),
            Expecting::Item(item) => {
                Ok(Reading { query: self.query.with_ask(item.ask(Spelling::name(word)?)), expecting: Expecting::Word })
            }
            Expecting::From => Ok(Reading { query: self.query, expecting: Expecting::To(Spelling::name(word)?) }),
            Expecting::To(from) => Ok(Reading {
                query: self.query.with_ask(Ask::Path(from, Spelling::name(word)?)),
                expecting: Expecting::Word,
            }),
        }
    }

    fn flagged(self, spelling: Spelling, word: &str) -> Result<Reading, QueryFailure> {
        match spelling {
            Spelling::Flag(expecting) => Ok(Reading { query: self.query, expecting }),
            Spelling::Unknown => Err(QueryFailure::Flag(word.to_owned())),
            Spelling::Term => Ok(Reading { query: self.query.with_term(word.parse()?), expecting: Expecting::Word }),
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

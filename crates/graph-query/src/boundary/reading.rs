use std::ffi::OsString;
use std::path::PathBuf;

use crate::boundary::query_failure::QueryFailure;
use crate::query::Query;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expecting {
    Word,
    Root,
    Role,
    Top,
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
    const FLAGS: [(&'static str, Expecting); 3] =
        [("--in", Expecting::Root), ("--role", Expecting::Role), ("--top", Expecting::Top)];

    fn flag(word: &str) -> Option<Expecting> {
        Self::FLAGS.into_iter().find(|(flag, _)| *flag == word).map(|(_, expecting)| expecting)
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
            Expecting::Root | Expecting::Role | Expecting::Top => Err(QueryFailure::Unfinished),
        }
    }

    fn asked(query: Query) -> Result<Query, QueryFailure> {
        query.asks().map(|_| ()).ok_or(QueryFailure::Empty).map(|()| query)
    }
}

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
        match (finished.expecting, finished.query.asks()) {
            (Expecting::Word, Some(_)) => Ok(finished.query),
            (Expecting::Word, None) => Err(QueryFailure::Empty),
            (Expecting::Root | Expecting::Role | Expecting::Top, _) => Err(QueryFailure::Unfinished),
        }
    }
}

impl Reading {
    const ROOT: &'static str = "--in";
    const ROLE: &'static str = "--role";
    const TOP: &'static str = "--top";

    fn next(self, word: &str) -> Result<Reading, QueryFailure> {
        let Reading { query, expecting } = self;
        Ok(match (expecting, word) {
            (Expecting::Word, Self::ROOT) => Reading { query, expecting: Expecting::Root },
            (Expecting::Word, Self::ROLE) => Reading { query, expecting: Expecting::Role },
            (Expecting::Word, Self::TOP) => Reading { query, expecting: Expecting::Top },
            (Expecting::Word, _) => Reading { query: query.with_term(word.parse()?), expecting },
            (Expecting::Root, _) => Reading { query: query.with_root(PathBuf::from(word)), expecting: Expecting::Word },
            (Expecting::Role, _) => Reading { query: query.with_role(word.parse()?), expecting: Expecting::Word },
            (Expecting::Top, _) => Reading { query: query.with_top(word.parse()?), expecting: Expecting::Word },
        })
    }
}

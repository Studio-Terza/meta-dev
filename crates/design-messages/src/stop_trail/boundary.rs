use std::fmt;

use design_model::{Awaited, Fate, Gist, Halt};
use invariants::Ignoring;
use newtype::{Arbitrary, Display, Newtype};
use store_ports::{Batch, Columns, Execute, Query, StoreFailure, Written};
use transcript_ports::SessionId;

use super::{StopTrail, Stopped};
use crate::boundary::ignored::Ignored;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally(i64);

#[derive(Clone, Debug, PartialEq, Eq, Columns)]
pub struct StopRow {
    session: SessionId,
    gist: Gist,
    read: Tally,
    quoted: Tally,
    kept: Tally,
    awaited: Awaited,
    halt: Halt,
    fate: Fate,
}

impl From<usize> for Tally {
    fn from(count: usize) -> Tally {
        Tally(i64::try_from(count).ignoring(Ignored::Oversized).unwrap_or(i64::MAX))
    }
}

impl StopRow {
    #[must_use]
    pub fn heard(session: SessionId, gist: Gist, [read, quoted, kept]: [Tally; 3]) -> StopRow {
        StopRow { session, gist, read, quoted, kept, awaited: Awaited::Unchecked, halt: Halt::Live, fate: Fate::Passed }
    }

    #[must_use]
    pub fn waited(self, awaited: Awaited, halt: Halt) -> StopRow {
        StopRow { awaited, halt, ..self }
    }

    #[must_use]
    pub fn ended(self, fate: Fate) -> StopRow {
        StopRow { fate, ..self }
    }
}

impl fmt::Display for StopRow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let StopRow { session, gist, read, quoted, kept, awaited, halt, fate } = self;
        write!(formatter, "{session}\t{gist}\t{read}\t{quoted}\t{kept}\t{awaited}\t{halt}\t{fate}")
    }
}

impl Batch for Stopped {
    type Failure = StoreFailure;

    fn written(self, statements: &mut impl Execute) -> Result<Written, StoreFailure> {
        let Stopped(StopRow { session, gist, read, quoted, kept, awaited, halt, fate }) = self;
        statements.written(store_ports::query!(
            r#"
INSERT INTO
  stops (
    session,
    gist,
    read,
    quoted,
    kept,
    awaited,
    halt,
    fate
  )
VALUES
  (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
"#,
            session: SessionId,
            gist: Gist,
            read: Tally,
            quoted: Tally,
            kept: Tally,
            awaited: Awaited,
            halt: Halt,
            fate: Fate
        ))
    }
}

impl Query for StopTrail {
    type Rows = Vec<StopRow>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<StopRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            StopRow,
            r#"
SELECT
  session,
  gist,
  read,
  quoted,
  kept,
  awaited,
  halt,
  fate
FROM
  stops
ORDER BY
  id
"#,
        ))
    }
}

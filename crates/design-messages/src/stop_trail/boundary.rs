use std::fmt;
use std::iter::Sum;

use invariants::Ignoring;
use newtype::{Arbitrary, Display, Listed, Newtype};
use store_ports::{Batch, Columns, Execute, Prepared, Query, StoreFailure};
use strum::{IntoStaticStr, VariantArray};
use transcript_ports::{Body, SessionId};

use super::{Overran, Overruns, StopTrail, Stopped};
use crate::boundary::ignored::Ignored;

#[derive(strum::Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Halt {
    Live,
    Stalled,
    Overrun,
}

#[derive(strum::Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Fate {
    Blocked,
    Passed,
}

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally(i64);

#[derive(Clone, Debug, PartialEq, Eq, Columns)]
pub struct StopRow {
    session: SessionId,
    shells: Tally,
    agents: Tally,
    halt: Halt,
    read: Tally,
    quoted: Tally,
    kept: Tally,
    fate: Fate,
    tokens: Tally,
    millis: Tally,
}

#[derive(Clone, Debug, PartialEq, Eq, Columns)]
struct OverrunRow {
    task: Body,
}

impl Sum for Tally {
    fn sum<Tallies: Iterator<Item = Tally>>(tallies: Tallies) -> Tally {
        Tally(tallies.fold(0_i64, |total, tally| total.saturating_add(tally.0)))
    }
}

impl From<usize> for Tally {
    fn from(count: usize) -> Tally {
        Tally(i64::try_from(count).ignoring(Ignored::Oversized).unwrap_or(i64::MAX))
    }
}

impl StopRow {
    #[must_use]
    pub fn ran(session: SessionId, [shells, agents]: [Tally; 2], halt: Halt) -> StopRow {
        let none = Tally::default();
        StopRow {
            session,
            shells,
            agents,
            halt,
            read: none,
            quoted: none,
            kept: none,
            fate: Fate::Passed,
            tokens: none,
            millis: none,
        }
    }

    #[must_use]
    pub fn searched(self, [read, quoted, kept, tokens]: [Tally; 4]) -> StopRow {
        StopRow { read, quoted, kept, tokens, ..self }
    }

    #[must_use]
    pub fn ended(self, fate: Fate, millis: Tally) -> StopRow {
        StopRow { fate, millis, ..self }
    }
}

impl OverrunRow {
    fn inserted(&self) -> Prepared<()> {
        let OverrunRow { task } = self;
        store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO overruns (task)
VALUES
  (?1)
"#,
            task: Body
        )
    }
}

impl fmt::Display for StopRow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let StopRow { session, shells, agents, halt, read, quoted, kept, fate, tokens, millis } = self;
        write!(formatter, "{session}\t{shells}\t{agents}\t{halt}\t{read}\t{quoted}\t{kept}\t{fate}\t{tokens}\t{millis}")
    }
}

impl Batch for Stopped {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        let Stopped(StopRow { session, shells, agents, halt, read, quoted, kept, fate, tokens, millis }) = self;
        statements.written(store_ports::query!(
            r#"
INSERT INTO
  stops (
    session,
    shells,
    agents,
    halt,
    read,
    quoted,
    kept,
    fate,
    tokens,
    millis
  )
VALUES
  (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
"#,
            session: SessionId,
            shells: Tally,
            agents: Tally,
            halt: Halt,
            read: Tally,
            quoted: Tally,
            kept: Tally,
            fate: Fate,
            tokens: Tally,
            millis: Tally
        ))
    }
}

impl Batch for Overran {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        let Overran(tasks) = self;
        let rows: Vec<OverrunRow> = tasks.into_iter().map(|task| OverrunRow { task }).collect();
        statements.written_all(rows.iter().map(OverrunRow::inserted))
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
  shells,
  agents,
  halt,
  read,
  quoted,
  kept,
  fate,
  tokens,
  millis
FROM
  stops
ORDER BY
  id
"#,
        ))
    }
}

impl Query for Overruns {
    type Rows = Vec<Body>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<Body>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            OverrunRow,
            r#"
SELECT
  task
FROM
  overruns
"#,
        ))?;
        Ok(rows.into_iter().map(|row| row.task).collect())
    }
}

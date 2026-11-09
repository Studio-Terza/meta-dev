use design_model::{JobId, Ordinal, Provenance, RunId, Segment};
use dispatch_ports::{Deadline, Pid, Started, Tokens, Usage, Wall};
use newtype::{Counter, Newtype};
use store_ports::{Batch, Columns, Query, Statements, StoreFailure, Written};
use transcript_ports::{Body, Uuid};

use super::{
    Delivery, FailuresReported, Flailed, FlailedSegments, Halted, JobClosed, JobFlailed, JobHalted, JobOpened,
    JobRetried, JobState, JobsInterrupted, LiveRuns, NextJob, NextRun, OpenJobs, Ran, Retold, RunEnded, RunSpawned,
    RunState, RunsOf, Spawned, StoredRun, Unplaced, UnplacedSegments, Unreported, UnreportedFailures, Unstructured,
};
use crate::{Account, Excerpt};

#[derive(Debug, Columns)]
struct UnreportedRow {
    job: JobId,
    account: Account,
}

#[derive(Debug, Columns)]
struct NextJobRow {
    next: JobId,
}

#[derive(Debug, Columns)]
struct OpenJobRow {
    id: JobId,
}

#[derive(Debug, Columns)]
struct NextRunRow {
    next: RunId,
}

#[derive(Debug, Columns)]
struct LiveRow {
    id: RunId,
    pid: Pid,
    started: Option<Started>,
    deadline: Wall,
}

#[derive(Debug, Columns)]
struct UnplacedRow {
    message: Uuid,
    ordinal: Ordinal,
    body: Body,
    run: Option<RunId>,
}

#[derive(Debug, Columns)]
struct FlailedRow {
    message: Uuid,
    ordinal: Ordinal,
    job: JobId,
    run: RunId,
}

#[derive(Debug, Columns)]
struct RanRow {
    id: RunId,
    state: RunState,
    input_tokens: Option<Tokens>,
    output_tokens: Option<Tokens>,
    cache_written_tokens: Option<Tokens>,
    cache_read_tokens: Option<Tokens>,
    reply: Option<Excerpt>,
}

#[derive(Clone, Copy, Debug)]
struct Spent {
    input: Option<Tokens>,
    output: Option<Tokens>,
    cache_written: Option<Tokens>,
    cache_read: Option<Tokens>,
}

impl From<Option<Usage>> for Spent {
    fn from(usage: Option<Usage>) -> Spent {
        let [input, output, cache_written, cache_read] = usage.map(|usage| usage.spent().map(Some)).unwrap_or_default();
        Spent { input, output, cache_written, cache_read }
    }
}

impl From<LiveRow> for StoredRun {
    fn from(row: LiveRow) -> StoredRun {
        StoredRun { run: row.id, spawned: Spawned::new(row.pid, row.started, Deadline::new(row.deadline)) }
    }
}

impl From<UnplacedRow> for Unplaced {
    fn from(row: UnplacedRow) -> Unplaced {
        Unplaced { segment: Segment::new(row.message, row.ordinal), body: row.body, flailed: row.run }
    }
}

impl From<FlailedRow> for Flailed {
    fn from(row: FlailedRow) -> Flailed {
        Flailed { segment: Segment::new(row.message, row.ordinal), job: row.job, run: row.run }
    }
}

impl From<UnreportedRow> for Unreported {
    fn from(row: UnreportedRow) -> Unreported {
        Unreported { job: row.job, account: row.account }
    }
}

impl From<RanRow> for Ran {
    fn from(row: RanRow) -> Ran {
        let usage = row
            .input_tokens
            .zip(row.output_tokens)
            .zip(row.cache_written_tokens.zip(row.cache_read_tokens))
            .map(|((input, output), (written, read))| Usage::new(input, output, written, read));
        Ran { run: row.id, state: row.state, usage, reply: row.reply }
    }
}

impl Batch for JobOpened {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let job = self.0;
        let open = JobState::Open;
        statements.written(store_ports::query!(
            r#"
INSERT INTO
  jobs (id, state)
VALUES
  (?1, ?2)
"#,
            job: JobId,
            open: JobState
        ))
    }
}

impl Batch for JobClosed {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let JobClosed { job, state } = self;
        statements.written(store_ports::query!(
            r#"
UPDATE
  jobs
SET
  state = ?2
WHERE
  id = ?1
"#,
            job: JobId,
            state: JobState
        ))
    }
}

impl Batch for JobHalted {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let JobHalted { job, halted, account, held, retold } = self;
        let closed = JobClosed::new(job, JobState::from(halted)).written(statements)?;
        let unreported = Delivery::Unreported;
        let recorded = statements.written(store_ports::query!(
            r#"
INSERT INTO
  failures (job, halted, account, delivery)
VALUES
  (?1, ?2, ?3, ?4)
"#,
            job: JobId,
            halted: Halted,
            account: Account,
            unreported: Delivery
        ))?;
        let holding = held.map(|held| held.written(statements)).transpose()?;
        let retelling = retold
            .into_iter()
            .map(|retold| retold.written(statements))
            .collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total([closed, recorded].into_iter().chain(holding).chain(retelling)))
    }
}

impl Batch for Retold {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let Retold { job, account } = self;
        statements.written(store_ports::query!(
            r#"
UPDATE
  failures
SET
  account = ?2
WHERE
  job = ?1
"#,
            job: JobId,
            account: Account
        ))
    }
}

impl Query for UnreportedFailures {
    type Rows = Vec<Unreported>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Unreported>, StoreFailure> {
        let unreported = Delivery::Unreported;
        let rows = statements.rows(store_ports::query!(
            UnreportedRow,
            r#"
SELECT
  job,
  account
FROM
  failures
WHERE
  delivery = ?1
ORDER BY
  job
"#,
            unreported: Delivery
        ))?;
        Ok(rows.into_iter().map(Unreported::from).collect())
    }
}

impl Batch for FailuresReported {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let reported = Delivery::Reported;
        let written = self
            .0
            .into_iter()
            .map(|job| {
                statements.written(store_ports::query!(
                    r#"
UPDATE
  failures
SET
  delivery = ?2
WHERE
  job = ?1
"#,
                    job: JobId,
                    reported: Delivery
                ))
            })
            .collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(written.into_iter()))
    }
}

impl Batch for JobsInterrupted {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let open = JobState::Open;
        let interrupted = JobState::Interrupted;
        statements.written(store_ports::query!(
            r#"
UPDATE
  jobs
SET
  state = ?2
WHERE
  state = ?1
"#,
            open: JobState,
            interrupted: JobState
        ))
    }
}

impl Query for OpenJobs {
    type Rows = Vec<JobId>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<JobId>, StoreFailure> {
        let open = JobState::Open;
        let rows = statements.rows(store_ports::query!(
            OpenJobRow,
            r#"
SELECT
  id
FROM
  jobs
WHERE
  state = ?1
ORDER BY
  id
"#,
            open: JobState
        ))?;
        Ok(rows.into_iter().map(|row| row.id).collect())
    }
}

impl Batch for RunSpawned {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let RunSpawned { run, job, spawned } = self;
        let pid = spawned.pid();
        let started = spawned.started().cloned();
        let deadline = AsRef::<Deadline>::as_ref(&spawned).into_inner();
        let running = RunState::Running;
        statements.written(store_ports::query!(
            r#"
INSERT INTO
  runs (id, job, pid, started, deadline, state)
VALUES
  (?1, ?2, ?3, ?4, ?5, ?6)
"#,
            run: RunId,
            job: JobId,
            pid: Pid,
            started: Option<Started>,
            deadline: Wall,
            running: RunState
        ))
    }
}

impl Batch for RunEnded {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let RunEnded { run, state, usage, reply } = self;
        let Spent { input, output, cache_written, cache_read } = Spent::from(usage);
        statements.written(store_ports::query!(
            r#"
UPDATE
  runs
SET
  state = ?2,
  input_tokens = ?3,
  output_tokens = ?4,
  cache_written_tokens = ?5,
  cache_read_tokens = ?6,
  reply = ?7
WHERE
  id = ?1
"#,
            run: RunId,
            state: RunState,
            input: Option<Tokens>,
            output: Option<Tokens>,
            cache_written: Option<Tokens>,
            cache_read: Option<Tokens>,
            reply: Option<Excerpt>
        ))
    }
}

impl Batch for JobFlailed {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let JobFlailed { run, segments } = self;
        let written = segments
            .iter()
            .map(|segment| {
                let message = AsRef::<Uuid>::as_ref(segment).clone();
                let ordinal = Ordinal::from(segment);
                statements.written(store_ports::query!(
                    r#"
INSERT INTO
  flailed (message, ordinal, run)
SELECT
  ?1,
  ?2,
  ?3
WHERE
  NOT EXISTS (
    SELECT
      1
    FROM
      structured
    WHERE
      structured.message = ?1
      AND structured.ordinal = ?2
  )
"#,
                    message: Uuid,
                    ordinal: Ordinal,
                    run: RunId
                ))
            })
            .collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(written.into_iter()))
    }
}

impl Batch for JobRetried {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let JobRetried { job, provenance } = self;
        let recorded = statements.written(store_ports::query!(
            r#"
INSERT INTO
  retries (job, provenance)
VALUES
  (?1, ?2)
"#,
            job: JobId,
            provenance: Provenance
        ))?;
        let cleared = statements.written(store_ports::query!(
            r#"
DELETE FROM
  flailed
WHERE
  run IN (
    SELECT
      id
    FROM
      runs
    WHERE
      job = ?1
  )
"#,
            job: JobId
        ))?;
        Ok(Written::total([recorded, cleared].into_iter()))
    }
}

impl Query for FlailedSegments {
    type Rows = Vec<Flailed>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Flailed>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            FlailedRow,
            r#"
SELECT
  flailed.message,
  flailed.ordinal,
  runs.job,
  flailed.run
FROM
  flailed
  JOIN runs ON runs.id = flailed.run
  JOIN messages ON messages.uuid = flailed.message
ORDER BY
  messages.at,
  flailed.message,
  flailed.ordinal
"#
        ))?;
        Ok(rows.into_iter().map(Flailed::from).collect())
    }
}

impl Query for NextJob {
    type Rows = JobId;

    fn rows(self, statements: &mut Statements<'_>) -> Result<JobId, StoreFailure> {
        let row = statements.one(store_ports::query!(
            NextJobRow,
            r#"
SELECT
  COALESCE(MAX(id), 0) + 1 AS "next!: i64"
FROM
  jobs
"#
        ))?;
        Ok(row.next)
    }
}

impl Query for NextRun {
    type Rows = RunId;

    fn rows(self, statements: &mut Statements<'_>) -> Result<RunId, StoreFailure> {
        let row = statements.one(store_ports::query!(
            NextRunRow,
            r#"
SELECT
  COALESCE(MAX(id), 0) + 1 AS "next!: i64"
FROM
  runs
"#
        ))?;
        Ok(row.next)
    }
}

impl Query for LiveRuns {
    type Rows = Vec<StoredRun>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<StoredRun>, StoreFailure> {
        let running = RunState::Running;
        let rows = statements.rows(store_ports::query!(
            LiveRow,
            r#"
SELECT
  id,
  pid,
  started,
  deadline
FROM
  runs
WHERE
  state = ?1
ORDER BY
  id
"#,
            running: RunState
        ))?;
        Ok(rows.into_iter().map(StoredRun::from).collect())
    }
}

impl Query for RunsOf {
    type Rows = Vec<Ran>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Ran>, StoreFailure> {
        let job = self.0;
        let rows = statements.rows(store_ports::query!(
            RanRow,
            r#"
SELECT
  id,
  state,
  input_tokens,
  output_tokens,
  cache_written_tokens,
  cache_read_tokens,
  reply
FROM
  runs
WHERE
  job = ?1
ORDER BY
  id
"#,
            job: JobId
        ))?;
        Ok(rows.into_iter().map(Ran::from).collect())
    }
}

impl Query for Unstructured {
    type Rows = Vec<Unplaced>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Unplaced>, StoreFailure> {
        let unplaced = UnplacedSegments.rows(statements)?;
        Ok(unplaced.into_iter().filter(|unplaced| unplaced.flailed.is_none()).collect())
    }
}

impl Query for UnplacedSegments {
    type Rows = Vec<Unplaced>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Unplaced>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            UnplacedRow,
            r#"
SELECT
  segments.message,
  segments.ordinal,
  segments.body,
  flailed.run AS "run?: i64"
FROM
  unplaced
  JOIN segments ON segments.message = unplaced.message
  AND segments.ordinal = unplaced.ordinal
  JOIN messages ON messages.uuid = unplaced.message
  LEFT JOIN flailed ON flailed.message = unplaced.message
  AND flailed.ordinal = unplaced.ordinal
ORDER BY
  messages.at,
  segments.message,
  segments.ordinal
"#
        ))?;
        Ok(rows.into_iter().map(Unplaced::from).collect())
    }
}

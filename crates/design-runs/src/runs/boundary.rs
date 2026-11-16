use design_model::{JobId, Ordinal, RunId, Segment};
use dispatch_ports::{Deadline, Pid, Started, Tokens, Usage, Wall};
use newtype::{Counter, Wrapping};
use store_ports::Record as _;
use store_ports::{Batch, Columns, Edge, Moved, Prepared, Query, Staged, Statements, StoreFailure, Written};
use transcript_ports::{Body, Instant, SessionId, Uuid};

use design_jobs::JobState;

use super::{
    Delivery, FailuresReported, Flailed, FlailedSegments, Halted, HeldBy, Hold, JobFlailed, JobHalted, JobOpened,
    JobRetried, JobsInterrupted, LiveRuns, NextJob, NextRun, OpenJobs, Ran, Retold, RunEnded, RunSpawned, RunState,
    RunsOf, Spawned, StoredRun, Unplaced, UnplacedSegments, Unreported, UnreportedFailures, Unstructured,
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

#[derive(Clone, Debug, PartialEq, Eq, Columns, store_ports::Record)]
#[record(table = "releases")]
pub struct Release {
    #[record(key)]
    job: JobId,
    #[record(generated)]
    released_at: Instant,
}

#[derive(Clone, Debug, PartialEq, Eq, Columns, store_ports::Record)]
#[record(table = "successions")]
pub struct Succession {
    #[record(key)]
    released: JobId,
    successor: JobId,
}

#[derive(Debug, Columns)]
struct OpenJobRow {
    id: JobId,
}

#[derive(Debug, Columns)]
struct DeliveredRow {
    job: JobId,
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
    session: SessionId,
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
        let [input, output, cache_written, cache_read] = usage.map_or_default(|reported| reported.spent().map(Some));
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
        Unplaced {
            segment: Segment::new(row.message, row.ordinal),
            body: row.body,
            session: row.session,
            flailed: row.run,
        }
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
        let JobOpened { job, covering } = self;
        let open = JobState::Open;
        let opened = statements.written(store_ports::query!(
            r#"
INSERT INTO
  jobs (id, state)
VALUES
  (?1, ?2)
"#,
            job: JobId,
            open: JobState
        ))?;
        let succeeded = covering
            .iter()
            .map(|segment| {
                let message = AsRef::<Uuid>::as_ref(segment).clone();
                let ordinal = Ordinal::from(segment);
                statements.written(store_ports::query!(
                    r#"
INSERT INTO
  successions (released, successor)
SELECT DISTINCT
  flailed.job,
  ?1
FROM
  flailed
  JOIN releases ON releases.job = flailed.job
WHERE
  flailed.message = ?2
  AND flailed.ordinal = ?3
  AND NOT EXISTS (
    SELECT
      1
    FROM
      successions
    WHERE
      successions.released = flailed.job
  )
"#,
                    job: JobId,
                    message: Uuid,
                    ordinal: Ordinal
                ))
            })
            .collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total([opened].into_iter().chain(succeeded)))
    }
}

impl Batch for JobHalted {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let JobHalted { job, halted, account, held, retold } = self;
        let closed = JobState::from(halted).closing(job).written(statements)?;
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
        let holding = held.map(|flailing| flailing.written(statements)).transpose()?;
        let retelling = retold
            .into_iter()
            .map(|again| again.written(statements))
            .collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total([closed, recorded].into_iter().chain(holding).chain(retelling)))
    }
}

impl Batch for Retold {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let Retold { job, account } = self;
        let recorded = statements.written(store_ports::query!(
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
        ))?;
        let moved = Delivery::Reported.shifted(Delivery::Unreported, job, statements)?;
        Ok(Written::total([recorded, moved].into_iter()))
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
        let written = self
            .0
            .into_iter()
            .map(|job| {
                Moved::<Delivery>::new(job, Edge::new(Delivery::Unreported, Delivery::Reported)).written(statements)
            })
            .collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(written.into_iter()))
    }
}

impl Batch for JobsInterrupted {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let moved = OpenJobs
            .rows(statements)?
            .into_iter()
            .map(|job| JobState::Interrupted.closing(job).written(statements))
            .collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(moved.into_iter()))
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
        let moved = Moved::<RunState>::new(run, Edge::new(RunState::Running, state)).written(statements)?;
        let recorded = statements.written(store_ports::query!(
            r#"
UPDATE
  runs
SET
  input_tokens = ?2,
  output_tokens = ?3,
  cache_written_tokens = ?4,
  cache_read_tokens = ?5,
  reply = ?6
WHERE
  id = ?1
"#,
            run: RunId,
            input: Option<Tokens>,
            output: Option<Tokens>,
            cache_written: Option<Tokens>,
            cache_read: Option<Tokens>,
            reply: Option<Excerpt>
        ))?;
        Ok(Written::total([moved, recorded].into_iter()))
    }
}

impl JobFlailed {
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
  flailed (message, ordinal, job, run)
SELECT
  ?1,
  ?2,
  (
    SELECT
      runs.job
    FROM
      runs
    WHERE
      runs.id = ?3
  ),
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
        let job = self.0;
        let released = statements.one(Release::created(&ReleaseNew { job }))?;
        let withdrawn = Delivery::Unreported.shifted(Delivery::Withdrawn, *released.key(), statements)?;
        Ok(Written::total([Written::new(1), withdrawn].into_iter()))
    }
}

impl Query for FlailedSegments {
    type Rows = Vec<Flailed>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Flailed>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            FlailedRow,
            r#"
SELECT
  held.message,
  held.ordinal,
  held.job,
  held.run
FROM
  held
  JOIN messages ON messages.uuid = held.message
ORDER BY
  messages.at,
  held.message,
  held.ordinal
"#
        ))?;
        Ok(rows.into_iter().map(Flailed::from).collect())
    }
}

impl Query for HeldBy {
    type Rows = Option<Hold>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Option<Hold>, StoreFailure> {
        let job = self.0;
        let rows = statements.rows(store_ports::query!(
            FlailedRow,
            r#"
SELECT
  held.message,
  held.ordinal,
  held.job,
  held.run
FROM
  held
  JOIN messages ON messages.uuid = held.message
WHERE
  held.job = ?1
ORDER BY
  messages.at,
  held.message,
  held.ordinal
"#,
            job: JobId
        ))?;
        let held: Vec<Flailed> = rows.into_iter().map(Flailed::from).collect();
        Ok(held.split_first().map(|(first, rest)| Hold { first: first.clone(), rest: rest.to_vec() }))
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
        Ok(unplaced.into_iter().filter(|segment| segment.flailed.is_none()).collect())
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
  messages.session,
  held.run AS "run?: i64"
FROM
  unplaced
  JOIN segments ON segments.message = unplaced.message
  AND segments.ordinal = unplaced.ordinal
  JOIN messages ON messages.uuid = unplaced.message
  LEFT JOIN held ON held.message = unplaced.message
  AND held.ordinal = unplaced.ordinal
ORDER BY
  messages.at,
  segments.message,
  segments.ordinal
"#
        ))?;
        Ok(rows.into_iter().map(Unplaced::from).collect())
    }
}

impl Staged for RunState {
    type Key = RunId;

    type State = RunState;

    fn moved(key: &RunId, from: RunState, to: RunState) -> Prepared<()> {
        store_ports::query!(
            r#"
UPDATE
  runs
SET
  state = ?3
WHERE
  id = ?1
  AND state = ?2
"#,
            key: RunId,
            from: RunState,
            to: RunState
        )
    }
}

impl Staged for Delivery {
    type Key = JobId;

    type State = Delivery;

    fn moved(key: &JobId, from: Delivery, to: Delivery) -> Prepared<()> {
        store_ports::query!(
            r#"
UPDATE
  failures
SET
  delivery = ?3
WHERE
  job = ?1
  AND delivery = ?2
"#,
            key: JobId,
            from: Delivery,
            to: Delivery
        )
    }
}

impl Delivery {
    fn shifted(self, to: Delivery, job: JobId, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let from = self;
        let delivered = statements.rows(store_ports::query!(
            DeliveredRow,
            r#"
SELECT
  job
FROM
  failures
WHERE
  job = ?1
  AND delivery = ?2
"#,
            job: JobId,
            from: Delivery
        ))?;
        let moved = delivered
            .into_iter()
            .map(|row| Moved::<Delivery>::new(row.job, Edge::new(from, to)).written(statements))
            .collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(moved.into_iter()))
    }
}

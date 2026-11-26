use design_model::{JobId, Ordinal, RunId, Segment};
use dispatch_ports::{Deadline, Pid, Started, Tokens, Usage, Wall};
use newtype::Wrapping;
use store_ports::Record as _;
use store_ports::{Batch, Columns, Edge, Execute, Moved, Prepared, Query, Staged, StoreFailure};
use transcript_ports::{Body, Instant, SessionId, Uuid};

use design_jobs::{JobState, OpenJobs};

use super::{
    Delivery, FailuresReported, Flailed, FlailedSegments, Halted, HeldBy, Hold, JobFlailed, JobHalted, JobOpened,
    JobRetried, JobsInterrupted, LiveRuns, NextJob, NextRun, Ran, Retold, RunEnded, RunReaped, RunSpawned, RunState,
    RunsOf, Spawned, StoredRun, Unplaced, UnplacedSegments, Unreported, UnreportedFailures, Unstructured, delivery,
    run,
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
        StoredRun {
            held: run::Running::held(row.id),
            spawned: Spawned::new(row.pid, row.started, Deadline::new(row.deadline)),
        }
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
        Unreported { held: delivery::Unreported::held(row.job), account: row.account }
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

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        let JobOpened { job, covering } = self;
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
        ))?;
        let succeeded = covering
            .iter()
            .map(|segment| {
                let message = AsRef::<Uuid>::as_ref(segment).clone();
                let ordinal = Ordinal::from(segment);
                store_ports::query!(
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
                )
            })
            .collect::<Vec<Prepared<()>>>();
        statements.written_all(succeeded)
    }
}

impl Batch for JobHalted {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        let JobHalted { job, halted, account, held, retold } = self;
        JobState::from(halted).closing(job).written(statements)?;
        let unreported = Delivery::Unreported;
        statements.written(store_ports::query!(
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
        held.map(|flailing| flailing.written(statements)).transpose()?;
        statements.written_all(retold)
    }
}

impl Batch for Retold {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
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
        ))?;
        let retold =
            Delivery::Reported.held(job, statements)?.map(|reported| delivery::Reported::held(reported).unreported());
        retold.map_or(Ok(()), |unreported| unreported.written(statements))
    }
}

impl Query for UnreportedFailures {
    type Rows = Vec<Unreported>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<Unreported>, StoreFailure> {
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

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        statements.written_all(self.0.into_iter().map(delivery::Unreported::reported))
    }
}

impl Batch for JobsInterrupted {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        let open = OpenJobs.rows(statements)?;
        statements.written_all(open.into_iter().map(design_jobs::job::Open::interrupted))
    }
}

impl Batch for RunSpawned {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
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

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        let RunEnded { run, state, usage, reply } = self;
        let Spent { input, output, cache_written, cache_read } = Spent::from(usage);
        Moved::<RunState>::new(run, Edge::new(RunState::Running, state)).written(statements)?;
        statements.written(store_ports::query!(
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
        ))
    }
}

impl JobFlailed {
    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        let JobFlailed { run, segments } = self;
        let written = segments
            .iter()
            .map(|segment| {
                let message = AsRef::<Uuid>::as_ref(segment).clone();
                let ordinal = Ordinal::from(segment);
                store_ports::query!(
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
                )
            })
            .collect::<Vec<Prepared<()>>>();
        statements.written_all(written)
    }
}

impl Batch for JobRetried {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        let job = self.0;
        let released = statements.one(Release::created(&ReleaseNew { job }))?;
        let unreported = Delivery::Unreported.held(*released.key(), statements)?;
        unreported.map_or(Ok(()), |held| delivery::Unreported::held(held).withdrawn().written(statements))
    }
}

impl Query for FlailedSegments {
    type Rows = Vec<Flailed>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<Flailed>, StoreFailure> {
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

    fn rows(self, statements: &mut impl Execute) -> Result<Option<Hold>, StoreFailure> {
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

    fn rows(self, statements: &mut impl Execute) -> Result<JobId, StoreFailure> {
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

    fn rows(self, statements: &mut impl Execute) -> Result<RunId, StoreFailure> {
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

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<StoredRun>, StoreFailure> {
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

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<Ran>, StoreFailure> {
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

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<Unplaced>, StoreFailure> {
        let unplaced = UnplacedSegments.rows(statements)?;
        Ok(unplaced.into_iter().filter(|segment| segment.flailed.is_none()).collect())
    }
}

impl Query for UnplacedSegments {
    type Rows = Vec<Unplaced>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<Unplaced>, StoreFailure> {
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
    fn held(self, job: JobId, statements: &mut impl Execute) -> Result<Option<JobId>, StoreFailure> {
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
        Ok(delivered.into_iter().next().map(|row| row.job))
    }
}

impl Batch for RunReaped {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        let RunReaped { held, ended } = self;
        held.moved(ended).written(statements)
    }
}

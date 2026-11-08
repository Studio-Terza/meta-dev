use design_model::{JobId, Ordinal, RunId, Segment};
use dispatch_ports::{Deadline, Pid, Started, Tokens, Usage, Wall};
use newtype::Newtype;
use store_ports::{Batch, Columns, Query, Statements, StoreFailure, Written};
use transcript_ports::{Authorship, Body, Uuid};

use super::{
    JobOpened, LiveRuns, NextJob, NextRun, Ran, RunEnded, RunSpawned, RunState, RunsOf, Spawned, StoredRun, Unplaced,
    Unstructured,
};

#[derive(Debug, Columns)]
struct NextJobRow {
    next: JobId,
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
}

#[derive(Debug, Columns)]
struct RanRow {
    id: RunId,
    state: RunState,
    input_tokens: Option<Tokens>,
    output_tokens: Option<Tokens>,
    cache_written_tokens: Option<Tokens>,
    cache_read_tokens: Option<Tokens>,
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
        Unplaced { segment: Segment::new(row.message, row.ordinal), body: row.body }
    }
}

impl From<RanRow> for Ran {
    fn from(row: RanRow) -> Ran {
        let usage = row
            .input_tokens
            .zip(row.output_tokens)
            .zip(row.cache_written_tokens.zip(row.cache_read_tokens))
            .map(|((input, output), (written, read))| Usage::new(input, output, written, read));
        Ran { run: row.id, state: row.state, usage }
    }
}

impl Batch for JobOpened {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let job = self.0;
        statements.written(store_ports::query!(
            r#"
INSERT INTO
  jobs (id)
VALUES
  (?1)
"#,
            job: JobId
        ))
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
        let RunEnded { run, state, usage } = self;
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
  cache_read_tokens = ?6
WHERE
  id = ?1
"#,
            run: RunId,
            state: RunState,
            input: Option<Tokens>,
            output: Option<Tokens>,
            cache_written: Option<Tokens>,
            cache_read: Option<Tokens>
        ))
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
  cache_read_tokens
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
        let owner = Authorship::Owner;
        let rows = statements.rows(store_ports::query!(
            UnplacedRow,
            r#"
SELECT
  segments.message,
  segments.ordinal,
  segments.body
FROM
  segments
  JOIN messages ON messages.uuid = segments.message
WHERE
  messages.author = ?1
  AND NOT EXISTS (
    SELECT
      1
    FROM
      span_edges
    WHERE
      span_edges.message = segments.message
      AND span_edges.ordinal = segments.ordinal
  )
  AND NOT EXISTS (
    SELECT
      1
    FROM
      senses
    WHERE
      senses.message = segments.message
      AND senses.ordinal = segments.ordinal
  )
  AND NOT EXISTS (
    SELECT
      1
    FROM
      conversational
    WHERE
      conversational.message = segments.message
      AND conversational.ordinal = segments.ordinal
  )
  AND NOT EXISTS (
    SELECT
      1
    FROM
      segment_links
    WHERE
      (
        segment_links.source_message = segments.message
        AND segment_links.source_ordinal = segments.ordinal
      )
      OR (
        segment_links.target_message = segments.message
        AND segment_links.target_ordinal = segments.ordinal
      )
  )
ORDER BY
  messages.at,
  segments.message,
  segments.ordinal
"#,
            owner: Authorship
        ))?;
        Ok(rows.into_iter().map(Unplaced::from).collect())
    }
}

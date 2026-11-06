use design_model::Node;
use design_model::{JobId, Ordinal, RunId, Segment};
use dispatch_ports::{Deadline, Pid, Spawned, Spend, Started, Tokens, Usage, Wall};
use newtype::{Counter, Newtype};
use store_ports::{Batch, Query, Statements, StoreFailure, Written};
use transcript_ports::{Authorship, Body, Uuid};

use super::{
    ContextNodes, JobClosed, JobOpened, JobState, LiveRuns, NextJob, NextRun, RunEnded, RunSpawned, RunState, SpentOn,
    StoredRun, Unplaced, Unstructured,
};
use crate::directions::Columns;
use crate::graph::NodeRow;

#[derive(Debug)]
struct LiveRow {
    id: i64,
    pid: i64,
    started: Option<String>,
    deadline: i64,
}

#[derive(Debug)]
struct UnplacedRow {
    message: String,
    ordinal: i64,
    body: String,
}

#[derive(Clone, Copy, Debug)]
struct SpentRow {
    input: i64,
    output: i64,
    cache_written: i64,
    cache_read: i64,
}

#[derive(Clone, Copy, Debug)]
struct UsageColumns {
    input: Option<i64>,
    output: Option<i64>,
    cache_written: Option<i64>,
    cache_read: Option<i64>,
}

impl ContextNodes {
    const LIMIT: i64 = 100;
}

impl TryFrom<LiveRow> for StoredRun {
    type Error = sqlx::Error;

    fn try_from(row: LiveRow) -> Result<StoredRun, sqlx::Error> {
        let pid = u32::try_from(row.pid).map_err(|negative| sqlx::Error::Decode(Box::new(negative)))?;
        let started = row.started.as_deref().map(NodeRow::decoded::<Started>).transpose()?;
        let spawned = Spawned::new(Pid::new(pid), started, Deadline::new(Wall::new(row.deadline)));
        Ok(StoredRun { run: RunId::new(row.id), spawned })
    }
}

impl TryFrom<UnplacedRow> for Unplaced {
    type Error = sqlx::Error;

    fn try_from(row: UnplacedRow) -> Result<Unplaced, sqlx::Error> {
        let ordinal = usize::try_from(row.ordinal).map_err(|negative| sqlx::Error::Decode(Box::new(negative)))?;
        Ok(Unplaced { segment: Segment::new(Uuid::new(row.message), Ordinal::new(ordinal)), body: Body::new(row.body) })
    }
}

impl From<Option<Usage>> for UsageColumns {
    fn from(usage: Option<Usage>) -> UsageColumns {
        let count = |spend: Spend| {
            usage.as_ref().map(|usage| i64::try_from(spend.spent_in(usage).into_inner()).unwrap_or(i64::MAX))
        };
        UsageColumns {
            input: count(Spend::Input),
            output: count(Spend::Output),
            cache_written: count(Spend::CacheWritten),
            cache_read: count(Spend::CacheRead),
        }
    }
}

impl Batch for JobOpened {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let job = self.job.into_inner();
        let state = JobState::Open.name();
        let opened = statements.written(sqlx::query!("INSERT INTO jobs (id, state) VALUES (?1, ?2)", job, state))?;
        let segments = self
            .segments
            .iter()
            .map(|segment| Columns::from(segment).assigned(job, statements))
            .collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total([opened].into_iter().chain(segments)))
    }
}

impl Batch for JobClosed {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let job = self.job.into_inner();
        let state = self.state.name();
        statements.written(sqlx::query!("UPDATE jobs SET state = ?2 WHERE id = ?1", job, state))
    }
}

impl Batch for RunSpawned {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let run = self.run.into_inner();
        let job = self.job.into_inner();
        let pid = i64::from(self.spawned.pid().into_inner());
        let started = self.spawned.started().map(ToString::to_string);
        let deadline = self.spawned.deadline().into_inner().into_inner();
        let state = RunState::Running.name();
        statements.written(sqlx::query!(
            "INSERT INTO runs (id, job, pid, started, deadline, state) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            run,
            job,
            pid,
            started,
            deadline,
            state
        ))
    }
}

impl Batch for RunEnded {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let run = self.run.into_inner();
        let state = self.state.name();
        let usage = UsageColumns::from(self.usage);
        statements.written(sqlx::query!(
            "UPDATE runs SET state = ?2, input_tokens = ?3, output_tokens = ?4, cache_written_tokens = ?5, cache_read_tokens = ?6 WHERE id = ?1",
            run,
            state,
            usage.input,
            usage.output,
            usage.cache_written,
            usage.cache_read
        ))
    }
}

impl Query for NextJob {
    type Rows = JobId;

    fn rows(self, statements: &mut Statements<'_>) -> Result<JobId, StoreFailure> {
        let rows = statements.rows(sqlx::query!(r#"SELECT COALESCE(MAX(id), 0) + 1 AS "next!: i64" FROM jobs"#))?;
        Ok(JobId::new(rows.first().map_or(1, |row| row.next)))
    }
}

impl Query for NextRun {
    type Rows = RunId;

    fn rows(self, statements: &mut Statements<'_>) -> Result<RunId, StoreFailure> {
        let rows = statements.rows(sqlx::query!(r#"SELECT COALESCE(MAX(id), 0) + 1 AS "next!: i64" FROM runs"#))?;
        Ok(RunId::new(rows.first().map_or(1, |row| row.next)))
    }
}

impl Query for LiveRuns {
    type Rows = Vec<StoredRun>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<StoredRun>, StoreFailure> {
        let running = RunState::Running.name();
        let rows = statements.rows(sqlx::query_as!(
            LiveRow,
            "SELECT id, pid, started, deadline FROM runs WHERE state = ?1 ORDER BY id",
            running
        ))?;
        Ok(rows.into_iter().map(StoredRun::try_from).collect::<Result<Vec<StoredRun>, sqlx::Error>>()?)
    }
}

impl Query for Unstructured {
    type Rows = Vec<Unplaced>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Unplaced>, StoreFailure> {
        let owner = Authorship::Owner.to_string();
        let rows = statements.rows(sqlx::query_as!(
            UnplacedRow,
            "SELECT segments.message, segments.ordinal, segments.body FROM segments JOIN messages ON messages.uuid = segments.message WHERE messages.author = ?1 AND NOT EXISTS (SELECT 1 FROM span_edges WHERE span_edges.message = segments.message AND span_edges.ordinal = segments.ordinal) AND NOT EXISTS (SELECT 1 FROM senses WHERE senses.message = segments.message AND senses.ordinal = segments.ordinal) AND NOT EXISTS (SELECT 1 FROM conversational WHERE conversational.message = segments.message AND conversational.ordinal = segments.ordinal) AND NOT EXISTS (SELECT 1 FROM segment_links WHERE (segment_links.source_message = segments.message AND segment_links.source_ordinal = segments.ordinal) OR (segment_links.target_message = segments.message AND segment_links.target_ordinal = segments.ordinal)) ORDER BY messages.at, segments.message, segments.ordinal",
            owner
        ))?;
        Ok(rows.into_iter().map(Unplaced::try_from).collect::<Result<Vec<Unplaced>, sqlx::Error>>()?)
    }
}

impl From<SpentRow> for Usage {
    fn from(row: SpentRow) -> Usage {
        let tokens = |count: i64| Tokens::new(u64::try_from(count).unwrap_or_default());
        Usage::new(tokens(row.input), tokens(row.output), tokens(row.cache_written), tokens(row.cache_read))
    }
}

impl Query for SpentOn {
    type Rows = Usage;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Usage, StoreFailure> {
        let job = self.0.into_inner();
        let rows = statements.rows(sqlx::query_as!(
            SpentRow,
            r#"SELECT COALESCE(SUM(input_tokens), 0) AS "input!: i64", COALESCE(SUM(output_tokens), 0) AS "output!: i64", COALESCE(SUM(cache_written_tokens), 0) AS "cache_written!: i64", COALESCE(SUM(cache_read_tokens), 0) AS "cache_read!: i64" FROM runs WHERE job = ?1"#,
            job
        ))?;
        Ok(rows.into_iter().next().map(Usage::from).unwrap_or_default())
    }
}

impl Query for ContextNodes {
    type Rows = Vec<Node>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Node>, StoreFailure> {
        NodeRow::context(statements, Self::LIMIT)
    }
}

use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::PathBuf;

use design_model::{
    Ending, JobId, Misstaged, NodeEdge, NodeId, Prefix, Project, Stage, Unnamed, Unnumbered, Unspecified, Unstated,
};
use design_store::{Refusal, Unpaid};
use dispatch_ports::{DispatchFailure, Stall};
use file_ports::FileFailure;
use local_config::ConfigFailure;
use newtype::Newtype;
use port_core::{ReleaseFailures, Snapshot};
use store_ports::{Busy, StoreFailure, Unlock};
use transcript_ports::ReadFailure;

use structurer::{Lapse, Misread, SegmentRef, Uncovered};

use super::output::Count;
use super::request::Unresolved;
use super::sequel::Sequel;
use crate::Outcome;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error(
        "usage: design [--workspace <dir>] (ingest [--dry-run] | show <node-id | message-ref | segment-ref | job <job-id>> | search [--all] <words...> | init <project> [--consumes <project>] | apply <file> | structure | refine <node-id> | specify <node-id> --artifact <lint|hook|tool|behaviour>:<name> --condition <text> | reopen <node-id> | drop <node-id> --reason <text> | role <node-id> <core|secondary> | hook (user-prompt-submit | stop | --settings) | view (current-decision | open-questions | outline) [<node-id>] | view (unplaced | notes) | import-notes <file> | retry <job-id>)"
    )]
    Usage,
    #[error("usage: project {0} cannot consume itself; --consumes names another project, one it builds on")]
    SelfConsuming(Project),
    #[error("the environment does not name {0:?}")]
    Unset(OsString),
    #[error("the workspace path {0:?} is not text")]
    Unreadable(PathBuf),
    #[error("cannot reach the transcripts: {0}")]
    Files(#[from] FileFailure),
    #[error("cannot read the transcripts: {0}")]
    Read(#[from] ReadFailure),
    #[error("cannot write the report: {0}")]
    Write(#[from] io::Error),
    #[error("{0}")]
    Unmatched(#[from] Unmatched),
    #[error("{0}")]
    Project(#[from] Unnamed),
    #[error("{0}")]
    Unnumbered(#[from] Unnumbered),
    #[error("no node n{} in the design store", .0.into_inner())]
    Unplaced(NodeId),
    #[error("node n{} is {ending}, itself or through a node it refines; view a current node", node.into_inner())]
    Ended { node: NodeId, ending: Ending },
    #[error("no run of job {0} in the design store")]
    Unrun(JobId),
    #[error("job {0} holds no flailed segments: it did not flail, or it was retried already; nothing was written")]
    Unflailed(JobId),
    #[error("no project {0} in the design store; run design init {0} in its repository first")]
    Unrooted(Project),
    #[error("no project is configured here; name a node, or run design init <project> in its repository first")]
    Unconfigured,
    #[error("cannot keep the local config: {0}")]
    Config(#[from] ConfigFailure),
    #[error("cannot read the directives file: {0}")]
    Directives(#[source] FileFailure),
    #[error("cannot read the directives: {0}")]
    Misread(#[from] Misread),
    #[error("cannot read the notes file: {0}")]
    Notes(#[source] FileFailure),
    #[error("project {0} already holds imported notes; nothing was written")]
    Reimported(Project),
    #[error(
        "{rulings} RULING lines resolve no owner words, neither a quote nor a cited owner message, or cite an owner message of several segments without quoting the decided words (an ambiguous anchor); quote or cite the owner on each and import again; nothing was written"
    )]
    Unanchored { rulings: Count },
    #[error("{lines} notes lines came in but {nodes} nodes came out")]
    Unreconciled { lines: Count, nodes: Count },
    #[error("{0}; nothing was written")]
    Unresolved(#[from] Unresolved),
    #[error("{0}; nothing was written")]
    Uncovered(#[from] Uncovered),
    #[error("{0}")]
    Unspecified(#[from] Unspecified),
    #[error("{0}")]
    Unstated(#[from] Unstated),
    #[error("n{} is {stage}, and that command does not start from {stage}; nothing was written", node.into_inner())]
    Misstaged { node: NodeId, stage: Stage },
    #[error(
        "n{} {} n{} would have a core node depend on a secondary one; nothing was written",
        edge.ends()[0].into_inner(),
        edge.relation(),
        edge.ends()[1].into_inner()
    )]
    Inverted { edge: NodeEdge },
    #[error("the design store lock was refused: {0}")]
    Busy(#[from] Busy),
    #[error("the design store failed: {0}")]
    Store(#[from] StoreFailure),
    #[error("cannot release the design store lock: {0}")]
    Unlock(#[from] Unlock),
    #[error("the structuring agent could not be run: {0}")]
    Dispatch(#[from] DispatchFailure),
    #[error("FLAILED: {flails}; the flailed segments are held for triage, and {sequel}")]
    Flailed { flails: Flails, sequel: Sequel },
    #[error(
        "STALLED: job {job} could not be run: {stall}; structure halted, and its segments are sent again on the next run"
    )]
    Stalled { job: JobId, stall: Stall },
    #[error("cannot hear the hook input: {0}")]
    Deaf(#[source] io::Error),
    #[error("cannot read the hook input: {0}")]
    Unheard(#[source] serde_json::Error),
    #[error("cannot render the hook settings: {0}")]
    Render(#[source] serde_json::Error),
    #[error("a resource was not released cleanly: {0}")]
    Released(#[from] ReleaseFailures),
    #[error("{command}; on the way out, a resource was not released cleanly either: {released}")]
    Unreleased { command: Box<Failure>, released: ReleaseFailures },
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("job {job} failed twice, the second time because {lapse} (design show job {job} prints its replies)")]
pub struct Flail {
    job: JobId,
    lapse: Lapse,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flails {
    first: Flail,
    rest: Vec<Flail>,
}

#[derive(Debug, thiserror::Error)]
pub enum Unmatched {
    #[error("no message in the design store starts with {0}; run design ingest first")]
    NoMatch(Prefix),
    #[error("several messages start with {0}; write more of the message id")]
    Ambiguous(Prefix),
}

impl Flail {
    #[must_use]
    pub fn new(job: JobId, lapse: Lapse) -> Flail {
        Flail { job, lapse }
    }
}

impl Flails {
    #[must_use]
    pub fn new(first: Flail, rest: Vec<Flail>) -> Flails {
        Flails { first, rest }
    }
}

impl fmt::Display for Flails {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.first)?;
        self.rest.iter().try_for_each(|flail| write!(formatter, "; {flail}"))
    }
}

impl From<Refusal> for Failure {
    fn from(refusal: Refusal) -> Failure {
        match refusal {
            Refusal::Cyclic(cycle) => Failure::Unresolved(Unresolved::from(cycle)),
            Refusal::Ancestral { old, by } => Failure::Unresolved(Unresolved::Ancestral { old, by }),
            Refusal::Inverted(inversion) => Failure::Inverted { edge: inversion.edge() },
            Refusal::Misstaged(misstaged) => Failure::from(misstaged),
            Refusal::Unpaid(unpaid) => Failure::from(unpaid),
            Refusal::Store(failure) => Failure::Store(failure),
        }
    }
}

impl From<Unpaid> for Failure {
    fn from(unpaid: Unpaid) -> Failure {
        Failure::Uncovered(Uncovered::new(
            SegmentRef::from(unpaid.first()),
            unpaid.rest().iter().map(SegmentRef::from).collect(),
        ))
    }
}

impl From<Misstaged> for Failure {
    fn from(misstaged: Misstaged) -> Failure {
        Failure::Misstaged { node: misstaged.node(), stage: misstaged.stage() }
    }
}

impl Failure {
    pub(crate) fn refusal(self) -> Result<Lapse, Failure> {
        match self {
            Failure::Unresolved(unresolved) => Ok(Lapse::refused(&unresolved)),
            Failure::Uncovered(uncovered) => Ok(Lapse::from(uncovered)),
            other => Err(other),
        }
    }

    pub fn settled(
        outcome: Result<Outcome, Failure>,
        released: Result<Snapshot, ReleaseFailures>,
    ) -> Result<Outcome, Failure> {
        match outcome {
            Ok(outcome) => released.map(|_snapshot| outcome).map_err(Failure::Released),
            Err(command) => Err(Failure::unreleasing(command, released)),
        }
    }

    fn unreleasing(command: Failure, released: Result<Snapshot, ReleaseFailures>) -> Failure {
        match released {
            Ok(_snapshot) => command,
            Err(released) => Failure::Unreleased { command: Box::new(command), released },
        }
    }
}

use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

use design_model::{Misstaged, NodeEdge, NodeId, Project, Stage, Unnamed, Unnumbered, Unspecified, Unstated};
use design_store::Refusal;
use file_ports::FileFailure;
use local_config::ConfigFailure;
use port_core::{ReleaseFailures, Snapshot};
use store_ports::{Busy, StoreFailure, Unlock};
use transcript_ports::{ReadFailure, Uuid};

use structurer::Misread;

use super::rendering::{Edge, Id};
use super::request::Unresolved;
use crate::Outcome;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error(
        "usage: design [--workspace <dir>] (ingest [--dry-run] | show <uuid | node-id> | search [--all] <words...> | init <project> [--consumes <project>] | apply <file> | refine <node-id> | specify <node-id> --artifact <lint|hook|tool|behaviour>:<name> --condition <text> | reopen <node-id> | drop <node-id> --reason <text> | role <node-id> <core|secondary>)"
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
    #[error("no message {0} in the design store; run design ingest first")]
    Missing(Uuid),
    #[error("{0}")]
    Project(#[from] Unnamed),
    #[error("{0}")]
    Unnumbered(#[from] Unnumbered),
    #[error("no node {} in the design store", Id::from(*.0))]
    Unplaced(NodeId),
    #[error("no project {0} in the design store; run design init {0} in its repository first")]
    Unrooted(Project),
    #[error("cannot keep the local config: {0}")]
    Config(#[from] ConfigFailure),
    #[error("cannot read the directives file: {0}")]
    Directives(#[source] FileFailure),
    #[error("cannot read the directives: {0}")]
    Misread(#[from] Misread),
    #[error("{0}; nothing was written")]
    Unresolved(#[from] Unresolved),
    #[error("{0}")]
    Unspecified(#[from] Unspecified),
    #[error("{0}")]
    Unstated(#[from] Unstated),
    #[error("{} is {stage}, and that command does not start from {stage}; nothing was written", Id::from(*node))]
    Misstaged { node: NodeId, stage: Stage },
    #[error("{} would have a core node depend on a secondary one; nothing was written", Edge::from(*edge))]
    Inverted { edge: NodeEdge },
    #[error("the design store lock was refused: {0}")]
    Busy(#[from] Busy),
    #[error("the design store failed: {0}")]
    Store(#[from] StoreFailure),
    #[error("cannot release the design store lock: {0}")]
    Unlock(#[from] Unlock),
    #[error("a resource was not released cleanly: {0}")]
    Released(#[from] ReleaseFailures),
    #[error("{command}; on the way out, a resource was not released cleanly either: {released}")]
    Unreleased { command: Box<Failure>, released: ReleaseFailures },
}

impl From<Refusal> for Failure {
    fn from(refusal: Refusal) -> Failure {
        match refusal {
            Refusal::Cyclic(cycle) => Failure::Unresolved(Unresolved::from(cycle)),
            Refusal::Inverted(inversion) => Failure::Inverted { edge: inversion.edge() },
            Refusal::Misstaged(misstaged) => Failure::from(misstaged),
            Refusal::Store(failure) => Failure::Store(failure),
        }
    }
}

impl From<Misstaged> for Failure {
    fn from(misstaged: Misstaged) -> Failure {
        Failure::Misstaged { node: misstaged.node(), stage: misstaged.stage() }
    }
}

impl Failure {
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

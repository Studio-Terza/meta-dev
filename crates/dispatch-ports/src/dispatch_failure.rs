use std::io;

use crate::finished::Unreadable;

#[derive(Debug, thiserror::Error)]
pub enum DispatchFailure {
    #[error("cannot make the agent's scratch directory: {0}")]
    Scratch(#[source] io::Error),
    #[error("cannot start the agent: {0}")]
    Spawn(#[source] io::Error),
    #[error("the agent was started without its standard streams")]
    Unpiped,
    #[error("cannot hand the agent its prompt: {0}")]
    Prompt(#[source] io::Error),
    #[error("cannot read the agent's output: {0}")]
    Collect(#[source] io::Error),
    #[error("the thread reading the agent's output ended abnormally")]
    Unjoined,
    #[error("{0}")]
    Unreadable(#[from] Unreadable),
    #[error("cannot look up a process start time: {0}")]
    Lookup(#[source] io::Error),
    #[error("cannot kill a process group: {0}")]
    Kill(#[source] io::Error),
}

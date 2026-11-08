use std::io;

#[derive(Debug, thiserror::Error)]
pub enum DispatchFailure {
    #[error("cannot make the agent's scratch directory: {0}")]
    Scratch(#[source] io::Error),
    #[error("cannot start the agent: {0}")]
    Spawn(#[source] io::Error),
    #[error("cannot make the pipes to the agent: {0}")]
    Pipe(#[source] io::Error),
    #[error("cannot hand the agent its prompt: {0}")]
    Prompt(#[source] io::Error),
    #[error("cannot read the agent's output: {0}")]
    Collect(#[source] io::Error),
    #[error("a thread exchanging the prompt and reply with the agent ended abnormally")]
    Unjoined,
    #[error("cannot look up a process start time: {0}")]
    Lookup(#[source] io::Error),
    #[error("cannot kill a process group: {0}")]
    Kill(#[source] io::Error),
}

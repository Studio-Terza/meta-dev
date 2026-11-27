#![deny(clippy::wildcard_enum_match_arm)]

mod agent_run;
mod bell;
mod clock;
mod dispatch;
mod dispatch_failure;
mod finished;
mod headless_claude;
mod job;
mod memory_clock;
mod pid;
mod program;
mod prompt;
mod replayed_dispatch;
mod shell;
mod started;
mod stderr;
mod system_clock;
mod system_shell;
mod wall;
mod watch;

pub use agent_run::AgentRun;
pub use bell::Bell;
pub use bell::Woke;
pub use clock::{Clock, ClockFiller, clock_ops};
pub use dispatch::{Dispatch, DispatchFiller, dispatch_ops};
pub use dispatch_failure::DispatchFailure;
pub(crate) use dispatch_failure::Fault;
pub(crate) use finished::{Errored, Unreadable};
pub use finished::{Finished, Output, Reply, Tokens, Usage};
pub use headless_claude::HeadlessClaude;
pub use job::{Job, Model};
pub use memory_clock::MemoryClock;
pub use pid::{Ended, Pid};
pub use program::Program;
pub use prompt::Prompt;
pub use replayed_dispatch::ReplayedDispatch;
pub use shell::{AnsweringShell, Call, Printed, Ran, Shell, ShellFiller, Spoken, shell_ops};
pub use started::Started;
pub use stderr::Stderr;
pub use system_clock::SystemClock;
pub use system_shell::SystemShell;
pub use wall::Due;
pub use wall::{Deadline, Moment, Pause, Timeout, Wall};
pub use watch::{Stall, Watch, Watched, Watching};

invariants::ignored! {
    pub(crate) enum Ignored {
        NotRunning => "a listing that does not read as a start time means no process holds the pid",
        NotAStamp => "any part that does not read as a start stamp field makes the whole text Unstarted",
        BeforeEpoch => "a clock set before the Unix epoch reads as the epoch",
        BeyondRange => "seconds past the i64 range saturate at its maximum",
        ManyRuns => "a replay running more agents than a pid counts gives the rest the last pid",
    }
}

#![deny(clippy::wildcard_enum_match_arm)]

mod agent;
mod agent_run;
mod bell;
mod clock;
mod dispatch;
mod dispatch_failure;
mod finished;
mod headless_claude;
mod job;
mod pid;
mod prompt;
mod shell;
mod started;
mod stderr;
mod system_clock;
mod system_shell;
mod wall;
mod watch;

pub use agent::Agent;
pub use agent_run::AgentRun;
pub use bell::{Bell, Woke};
pub use clock::{Clock, ClockFiller, clock_ops};
pub use dispatch::{Dispatch, DispatchFiller, dispatch_ops};
pub use dispatch_failure::{DispatchFailure, Fault};
pub use finished::{Errored, Finished, Output, Reply, Tokens, Told, Unreadable, Usage};
pub use headless_claude::HeadlessClaude;
pub use job::{Job, Model};
pub use pid::{Ended, Pid};
pub use prompt::Prompt;
pub use shell::{Call, Printed, Ran, Shell, ShellFiller, shell_ops};
pub use started::Started;
pub use stderr::Stderr;
pub use system_clock::SystemClock;
pub use system_shell::SystemShell;
pub use wall::{Deadline, Due, Moment, Pause, Timeout, Wall};
pub use watch::{Stall, Stalled, Watch, Watched, Watching};

invariants::ignored! {
    pub(crate) enum Ignored {
        NotRunning => "a listing that does not read as a start time means no process holds the pid",
        NotAStamp => "any part that does not read as a start stamp field makes the whole text Unstarted",
        BeforeEpoch => "a clock set before the Unix epoch reads as the epoch",
        BeyondRange => "seconds past the i64 range saturate at its maximum",
    }
}

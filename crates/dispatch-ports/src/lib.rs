#![deny(clippy::wildcard_enum_match_arm)]

mod agent;
mod agent_run;
mod bell;
mod boundary;
mod clock;
mod dispatch;
mod dispatch_failure;
mod finished;
mod headless_claude;
mod job;
mod pid;
mod scripted;
mod shell;
mod started;
mod stderr;
mod stepped;
mod system_clock;
mod system_shell;
mod wall;
mod watch;

pub use agent::Agent;
pub use agent_run::AgentRun;
pub use bell::Bell;
use bell::Woke;
pub use boundary::{Count, Prompt};
pub use clock::{Clock, ClockFiller, clock_ops};
pub use dispatch::{Dispatch, DispatchFiller, dispatch_ops};
pub use dispatch_failure::DispatchFailure;
use dispatch_failure::Fault;
use finished::{Errored, Unreadable};
pub use finished::{Finished, Output, Reply, Tokens, Usage};
pub use headless_claude::HeadlessClaude;
pub use job::{Job, Model};
use pid::Ended;
pub use pid::Pid;
pub use scripted::{Ringing, Scripted};
use shell::Printed;
pub use shell::{Call, Ran, Shell, ShellFiller, shell_ops};
pub use started::Started;
pub use stderr::Stderr;
pub use stepped::{Oversleep, Stepped, Wait};
pub use system_clock::SystemClock;
pub use system_shell::SystemShell;
pub use wall::{Deadline, Due, Moment, Pause, Timeout, Wall};
use watch::Watched;
pub use watch::{Stall, Watch, Watching};

invariants::ignored! {
    pub(crate) enum Ignored {
        NotRunning => "a listing that does not read as a start time means no process holds the pid",
        NotAStamp => "any part that does not read as a start stamp field makes the whole text Unstarted",
        BeforeEpoch => "a clock set before the Unix epoch reads as the epoch",
        BeyondRange => "seconds past the i64 range saturate at its maximum",
    }
}

#![deny(clippy::wildcard_enum_match_arm)]

mod clock;
mod dispatch_failure;
mod shell;
mod system_clock;
mod system_shell;
mod wall;

pub use clock::{Clock, ClockFiller, clock_ops};
pub use dispatch_failure::DispatchFailure;
pub use shell::{Call, Printed, Ran, Shell, ShellFiller, shell_ops};
pub use system_clock::SystemClock;
pub use system_shell::SystemShell;
pub use wall::{Deadline, Due, Pause, Timeout, Wall};

invariants::ignored! {
    pub(crate) enum Ignored {
        BeforeEpoch => "a clock set before the Unix epoch reads as the epoch",
        BeyondRange => "seconds past the i64 range saturate at its maximum",
    }
}

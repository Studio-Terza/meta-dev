#![deny(clippy::wildcard_enum_match_arm)]

mod dispatch_failure;
mod ignored;
mod shell;
mod system_shell;

pub use dispatch_failure::DispatchFailure;
pub use shell::{Call, Printed, Ran, Shell, ShellFiller, shell_ops};
pub use system_shell::SystemShell;

mod changed;
mod compile_times;
pub mod github;
mod history;
mod install_tools;
mod journal;
mod libtest_seconds;
mod logged;
mod measured;
pub mod mutation;
mod reach_scope;
mod seconds;
mod timed;

use std::ffi::OsString;
use std::io::Write;

use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};

pub use compile_times::CompileTimes;
pub use history::History;
pub use install_tools::InstallTools;
pub use libtest_seconds::LibtestSeconds;
pub use logged::Logged;
pub use reach_scope::ReachScope;
pub use timed::Timed;

pub trait Step: Sized {
    fn from_arguments(arguments: &[OsString]) -> Result<Self, Failure>;

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure>;
}

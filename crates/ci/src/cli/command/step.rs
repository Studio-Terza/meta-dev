mod arguments;
mod changed;
pub mod clippy_targets;
pub mod cold_size;
mod compile_times;
pub mod github;
pub mod graph_path;
mod install_tools;
mod journal;
mod libtest_seconds;
mod logged;
mod measured;
pub mod mutation;
pub mod ratchet;
mod revision;
mod seconds;
pub mod sqlite;
mod stage;
mod timed;

use std::ffi::OsString;

use crate::cli::command::Telling;
use crate::cli::command::failure::Failure;
use crate::cli::ports::Ports;
pub(crate) use arguments::{Arguments, Ended, Flag, Journaled, Location, Parsing, Recording, Utf8, Word};

pub use compile_times::CompileTimes;
pub use install_tools::InstallTools;
pub use libtest_seconds::LibtestSeconds;
pub use logged::Logged;
pub use timed::Timed;

pub trait Step: Sized {
    fn from_arguments(arguments: Vec<OsString>) -> Result<Self, Failure<'static>>;

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>>;
}

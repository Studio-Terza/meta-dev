mod arguments;
mod changed;
pub(crate) mod clippy_targets;
pub(crate) mod cold_size;
mod compile_times;
pub(crate) mod github;
pub(crate) mod graph_path;
mod install_tools;
mod journal;
mod libtest_seconds;
mod logged;
mod measured;
pub(crate) mod mutation;
pub(crate) mod ratchet;
mod revision;
mod seconds;
pub(crate) mod sqlite;
mod stage;
mod timed;

use std::ffi::OsString;

use crate::cli::command::Telling;
use crate::cli::command::failure::Failure;
use crate::cli::ports::Ports;
pub(crate) use arguments::{Arguments, Ended, Flag, Journaled, Location, Parsing, Recording, Utf8, Word};

pub(crate) use compile_times::CompileTimes;
pub(crate) use install_tools::InstallTools;
pub(crate) use libtest_seconds::LibtestSeconds;
pub(crate) use logged::Logged;
pub(crate) use timed::Timed;

pub(crate) trait Step: Sized {
    fn from_arguments(arguments: Vec<OsString>) -> Result<Self, Failure<'static>>;

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>>;
}

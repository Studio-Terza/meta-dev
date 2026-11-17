mod arguments;
mod changed;
pub mod clippy_targets;
pub mod cold_size;
mod compile_times;
pub mod github;
pub mod graph_path;
mod history;
mod install_tools;
mod journal;
mod libtest_seconds;
mod logged;
mod measured;
pub mod mutation;
pub mod ratchet;
mod reach_scope;
mod revision;
mod seconds;
pub mod sqlite;
mod stage;
mod timed;
mod tools_key;

use std::ffi::OsString;

use crate::cli::command::Telling;
use crate::cli::command::failure::Failure;
use crate::cli::ports::Ports;
pub(crate) use arguments::{Arguments, Flag};

pub use compile_times::CompileTimes;
pub use history::History;
pub use install_tools::InstallTools;
pub use libtest_seconds::LibtestSeconds;
pub use logged::Logged;
pub use reach_scope::ReachScope;
pub use timed::Timed;
pub use tools_key::ToolsKey;

pub trait Step: Sized {
    fn from_arguments(arguments: Vec<OsString>) -> Result<Self, Failure<'static>>;

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>>;
}

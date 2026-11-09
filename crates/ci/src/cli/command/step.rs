mod arguments;
mod branch;
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
use std::io::Write;

use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};
pub(crate) use arguments::Arguments;

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

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>>;
}

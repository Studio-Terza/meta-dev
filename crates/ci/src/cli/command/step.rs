mod libtest_seconds;
mod measured;
mod reach_scope;
mod seconds;
mod timed;

use std::ffi::OsString;
use std::io::Write;

use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};

pub use libtest_seconds::LibtestSeconds;
pub use reach_scope::ReachScope;
pub use timed::Timed;

pub trait Step: Sized {
    fn from_arguments(arguments: &[OsString]) -> Result<Self, Failure>;

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure>;
}

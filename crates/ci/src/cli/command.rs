mod failure;
mod step;

use std::ffi::OsString;
use std::io::Write;

use step::{LibtestSeconds, ReachScope, Step, Timed};

use super::ports::{Exit, Ports};
use failure::Failure;

#[derive(Debug)]
pub enum Command {
    Timed(Timed),
    LibtestSeconds(LibtestSeconds),
    Reach(ReachScope),
}

impl Command {
    const TIMED: &'static str = "timed";
    const LIBTEST_SECONDS: &'static str = "libtest-seconds";
    const REACH: &'static str = "reach";

    pub fn parse(mut arguments: impl Iterator<Item = OsString>) -> Result<Command, Failure> {
        let name = arguments.next().and_then(|name| name.into_string().ok());
        let rest: Vec<OsString> = arguments.collect();
        match name.as_deref() {
            Some(Self::TIMED) => Timed::from_arguments(&rest).map(Command::Timed),
            Some(Self::LIBTEST_SECONDS) => LibtestSeconds::from_arguments(&rest).map(Command::LibtestSeconds),
            Some(Self::REACH) => ReachScope::from_arguments(&rest).map(Command::Reach),
            _ => Err(Failure::Usage),
        }
    }

    pub fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        match self {
            Command::Timed(step) => step.carry_out(ports, out),
            Command::LibtestSeconds(step) => step.carry_out(ports, out),
            Command::Reach(step) => step.carry_out(ports, out),
        }
    }
}

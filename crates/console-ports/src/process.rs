use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{self, Stderr, Stdout};
use std::process::ExitCode;

use crate::{Answer, Environment, Heard};
use port_core::Real;

#[derive(Clone, Copy, Debug, Default)]
pub struct Process;

impl Real for Process {}

impl Process {
    pub fn stream<Ended>(respond: impl FnOnce(Vec<OsString>, &mut Stdout, &mut Stderr) -> Ended) -> Ended {
        respond(env::args_os().skip(1).collect(), &mut io::stdout(), &mut io::stderr())
    }

    pub fn tell(parse: impl FnOnce(Vec<OsString>) -> Answer) -> ExitCode {
        Process::stream(|arguments, out, err| parse(arguments).written(out, err))
    }
}

impl Environment for Process {
    fn variable(&self, name: &OsStr) -> Option<OsString> {
        env::var_os(name)
    }

    fn input(&self) -> io::Result<Heard> {
        io::read_to_string(io::stdin()).map(Heard::new)
    }
}

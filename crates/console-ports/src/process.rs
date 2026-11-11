use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{self, Stderr, Stdout};

use crate::{Environment, Heard};

#[derive(Clone, Copy, Debug, Default)]
pub struct Process;

impl Process {
    pub fn answer<Answer>(respond: impl FnOnce(Vec<OsString>, &mut Stdout, &mut Stderr) -> Answer) -> Answer {
        respond(env::args_os().skip(1).collect(), &mut io::stdout(), &mut io::stderr())
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

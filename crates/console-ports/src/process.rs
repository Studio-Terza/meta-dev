mod ignored;

use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{self, Stderr, Stdout, Write};
use std::process::ExitCode;

use crate::process::ignored::Ignored;
use crate::{Answer, Complaint, Environment, Heard, Shown};
use invariants::Ignoring;
use port_core::Real;

#[derive(Real, Clone, Copy, Debug, Default)]
pub struct Process;

impl Process {
    pub fn stream<Ended>(respond: impl FnOnce(Vec<OsString>, &mut Stdout, &mut Stderr) -> Ended) -> Ended {
        respond(env::args_os().skip(1).collect(), &mut io::stdout(), &mut io::stderr())
    }

    pub fn tell(parse: impl FnOnce(Vec<OsString>) -> Answer) -> ExitCode {
        Process::written(&parse(env::args_os().skip(1).collect()))
    }

    fn written(answer: &Answer) -> ExitCode {
        let shown: &Shown = answer.as_ref();
        let complaint: &Complaint = answer.as_ref();
        let written = io::stdout()
            .write_all(AsRef::<str>::as_ref(shown).as_bytes())
            .and_then(|()| io::stderr().write_all(AsRef::<str>::as_ref(complaint).as_bytes()));
        written.ignoring(Ignored::Unwritable).map_or(ExitCode::FAILURE, |()| answer.code())
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

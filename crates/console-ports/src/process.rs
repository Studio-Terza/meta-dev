mod ignored;

use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{self, Stderr, Stdout, Write};
use std::process::ExitCode;

use crate::process::ignored::Ignored;
use crate::{Answer, Complaint, Environment, Heard, Shown, Tale, Teller};
use invariants::Ignoring;

#[derive(Clone, Copy, Debug, Default)]
pub struct Process;

impl Process {
    pub fn stream<Ended>(respond: impl FnOnce(Vec<OsString>, &mut Stdout, &mut Stderr) -> Ended) -> Ended {
        respond(env::args_os().skip(1).collect(), &mut io::stdout(), &mut io::stderr())
    }

    pub fn tell<Told: Teller>(parse: impl FnOnce(Vec<OsString>) -> Told) -> ExitCode {
        let teller = parse(env::args_os().skip(1).collect());
        let mut tale = teller.told();
        let printed = tale.by_ref().try_for_each(|line| io::stdout().write_all(AsRef::<str>::as_ref(&line).as_bytes()));
        printed.ignoring(Ignored::Unwritable).map_or(ExitCode::FAILURE, |()| Process::written(&tale.ended()))
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

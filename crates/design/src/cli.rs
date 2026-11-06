mod failure;
mod request;

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use console_ports::Environment;
use file_ports::Files;
use port_core::Run;
use store_ports::Sqlite;

use crate::Outcome;

pub use failure::Failure;
use request::Request;

#[derive(Debug)]
pub struct Cli<'environment, E: Environment, F: Files, O> {
    environment: &'environment E,
    files: F,
    opened: O,
}

impl<'environment, E: Environment, F: Files, O> Cli<'environment, E, F, O> {
    pub fn new(environment: &'environment E, files: F, opened: O) -> Cli<'environment, E, F, O> {
        Cli { environment, files, opened }
    }

    pub fn rooted(self, arguments: impl Iterator<Item = OsString>, out: &mut impl Write) -> Result<Outcome, Failure>
    where
        O: FnOnce(PathBuf) -> Sqlite,
    {
        let run = Run::root();
        let outcome = self.report(&run, arguments, out);
        Failure::settled(outcome, run.finish().settled())
    }

    pub fn report(
        self,
        run: &Run,
        arguments: impl Iterator<Item = OsString>,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure>
    where
        O: FnOnce(PathBuf) -> Sqlite,
    {
        let request = Request::parsed(arguments)?;
        let located = request.located(self.environment)?;
        let transcripts = run.port(located.transcripts(self.files));
        let store = run.port(request.persistence().store(located, self.opened));
        request.command().written(&transcripts, &store, out)
    }
}

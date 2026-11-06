mod failure;
mod request;

use std::ffi::OsString;
use std::io::Write;

use console_ports::Environment;
use file_ports::Files;
use port_core::Run;
use transcript_ports::ClaudeCodeDir;

use crate::Outcome;

pub use failure::Failure;
use request::Request;

#[derive(Debug)]
pub struct Cli<'environment, E: Environment, F: Files> {
    environment: &'environment E,
    files: F,
}

impl<'environment, E: Environment, F: Files> Cli<'environment, E, F> {
    pub fn new(environment: &'environment E, files: F) -> Cli<'environment, E, F> {
        Cli { environment, files }
    }

    pub fn rooted(self, arguments: impl Iterator<Item = OsString>, out: &mut impl Write) -> Result<Outcome, Failure> {
        let run = Run::root();
        let outcome = self.report(&run, arguments, out);
        run.finish();
        outcome
    }

    pub fn report(
        self,
        run: &Run,
        arguments: impl Iterator<Item = OsString>,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        let request = Request::parsed(arguments)?;
        let directory = request.directory(self.environment)?;
        let port = run.port(ClaudeCodeDir::new(self.files, directory));
        request.command().written(&port, out)
    }
}

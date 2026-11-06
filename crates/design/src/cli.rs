mod failure;
mod printed;
mod request;
mod streams;

use std::ffi::OsString;
use std::io::Write;
use std::process::ExitCode;

use console_ports::Environment;
use file_ports::Files;
use port_core::{Rooting, Run};
use transcript_ports::ClaudeCodeDir;

pub use failure::Failure;
pub use printed::Printed;
use request::Request;
pub use streams::Streams;

#[derive(Debug)]
pub struct Cli<'environment, E: Environment, F: Files> {
    environment: &'environment E,
    files: F,
}

impl<'environment, E: Environment, F: Files> Cli<'environment, E, F> {
    pub fn new(environment: &'environment E, files: F) -> Cli<'environment, E, F> {
        Cli { environment, files }
    }

    #[must_use]
    pub fn rooted(
        self,
        rooting: Rooting,
        arguments: impl Iterator<Item = OsString>,
        streams: &mut Streams<impl Write, impl Write>,
    ) -> ExitCode {
        let outcome = Run::root(rooting).map_err(Failure::from).and_then(|run| {
            let reported = self.report(&run, arguments, streams.out());
            run.finish();
            reported
        });
        outcome.map_or_else(|failure| streams.refused(&failure), |_printed| ExitCode::SUCCESS)
    }

    pub fn report(
        self,
        run: &Run,
        arguments: impl Iterator<Item = OsString>,
        out: &mut impl Write,
    ) -> Result<Printed, Failure> {
        let request = Request::parsed(arguments)?;
        let directory = request.directory(self.environment)?;
        let port = run.port(ClaudeCodeDir::new(self.files, directory));
        request.command().written(&port, out)
    }
}

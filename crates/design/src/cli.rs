mod failure;
pub(crate) mod output;
mod request;

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use console_ports::Environment;
use dispatch_ports::{ClockFiller, DispatchFiller};
use file_ports::Files;
use port_core::Run;
use store_ports::Sqlite;

use crate::Outcome;

pub use failure::{Failure, Unmatched};
pub use request::Unresolved;
use request::{Crew, Ground, Kept, Request};

#[derive(Debug)]
pub struct Agents<H, C> {
    hired: H,
    clock: C,
}

#[derive(Debug)]
pub struct Cli<'environment, E: Environment, F: Files, O, H, C> {
    environment: &'environment E,
    files: F,
    opened: O,
    agents: Agents<H, C>,
}

impl<H, C: ClockFiller> Agents<H, C> {
    pub fn new(hired: H, clock: C) -> Agents<H, C> {
        Agents { hired, clock }
    }
}

impl<'environment, E: Environment, F: Files, O, H, C: ClockFiller> Cli<'environment, E, F, O, H, C> {
    pub fn new(
        environment: &'environment E,
        files: F,
        opened: O,
        agents: Agents<H, C>,
    ) -> Cli<'environment, E, F, O, H, C> {
        Cli { environment, files, opened, agents }
    }

    pub fn rooted<D: DispatchFiller>(
        self,
        arguments: impl Iterator<Item = OsString>,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure>
    where
        O: FnOnce(PathBuf) -> Sqlite,
        H: FnOnce(PathBuf) -> D,
    {
        let run = Run::root();
        let outcome = self.report(&run, arguments, out);
        Failure::settled(outcome, run.finish().settled())
    }

    pub fn report<D: DispatchFiller>(
        self,
        run: &Run,
        arguments: impl Iterator<Item = OsString>,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure>
    where
        O: FnOnce(PathBuf) -> Sqlite,
        H: FnOnce(PathBuf) -> D,
    {
        let request = Request::parsed(arguments)?;
        let located = request.located(self.environment, &self.files)?;
        let store = run.port(request.opened(&located, self.opened));
        let dispatch = run.port((self.agents.hired)(located.kept(Kept::Agents)));
        let clock = run.port(self.agents.clock);
        request.command().written(Ground::new(run, self.files, located, &store), &Crew::new(&dispatch, &clock), out)
    }
}

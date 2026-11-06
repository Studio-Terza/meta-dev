mod failure;
mod rendering;
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

pub use failure::Failure;
pub use request::Unresolved;
use request::{Crew, Ground, Request};

#[derive(Debug)]
pub struct Agents<D, C> {
    dispatch: D,
    clock: C,
}

#[derive(Debug)]
pub struct Cli<'environment, E: Environment, F: Files, O, D, C> {
    environment: &'environment E,
    files: F,
    opened: O,
    agents: Agents<D, C>,
}

impl<D: DispatchFiller, C: ClockFiller> Agents<D, C> {
    pub fn new(dispatch: D, clock: C) -> Agents<D, C> {
        Agents { dispatch, clock }
    }
}

impl<'environment, E: Environment, F: Files, O, D: DispatchFiller, C: ClockFiller> Cli<'environment, E, F, O, D, C> {
    pub fn new(
        environment: &'environment E,
        files: F,
        opened: O,
        agents: Agents<D, C>,
    ) -> Cli<'environment, E, F, O, D, C> {
        Cli { environment, files, opened, agents }
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
        let located = request.located(self.environment, &self.files)?;
        let store = run.port(request.opened(&located, self.opened));
        let dispatch = run.port(self.agents.dispatch);
        let clock = run.port(self.agents.clock);
        request.command().written(Ground::new(run, self.files, located, &store), &Crew::new(&dispatch, &clock), out)
    }
}

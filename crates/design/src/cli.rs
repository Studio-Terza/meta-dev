mod count;
mod failure;
pub(crate) mod outcome;
mod request;
pub(crate) mod uttered;

use std::ffi::OsString;
use std::path::PathBuf;

use console_ports::Environment;
use file_ports::Files;
use ports::dispatch::{ClockFiller, DispatchFiller, ShellFiller};
use ports::port::Run;
use ports::store::Opens;

pub use count::Count;
pub use failure::Failure;
use request::{Crew, Ground, Kept, Request};
use uttered::{Settled, Uttered};

#[derive(Debug)]
pub struct Agents<H, C, G> {
    hired: H,
    clock: C,
    shell: G,
}

#[derive(Debug)]
pub struct Cli<'environment, E: Environment, F: Files, O, H, C, G> {
    environment: &'environment E,
    files: F,
    opened: O,
    agents: Agents<H, C, G>,
}

impl<H, C: ClockFiller, G: ShellFiller> Agents<H, C, G> {
    pub fn new(hired: H, clock: C, shell: G) -> Agents<H, C, G> {
        Agents { hired, clock, shell }
    }
}

impl<'environment, E: Environment, F: Files, O, H, C: ClockFiller, G: ShellFiller> Cli<'environment, E, F, O, H, C, G> {
    pub fn new(
        environment: &'environment E,
        files: F,
        opened: O,
        agents: Agents<H, C, G>,
    ) -> Cli<'environment, E, F, O, H, C, G> {
        Cli { environment, files, opened, agents }
    }

    pub fn rooted<D: DispatchFiller>(self, arguments: impl Iterator<Item = OsString>) -> Uttered<Settled>
    where
        O: Opens,
        H: FnOnce(PathBuf) -> D,
    {
        let run = Run::root();
        let uttered = self.report(&run, arguments);
        uttered.settled(run.finish().settled())
    }

    pub fn report<D: DispatchFiller>(self, run: &Run, arguments: impl Iterator<Item = OsString>) -> Uttered
    where
        O: Opens,
        H: FnOnce(PathBuf) -> D,
    {
        match self.answered(run, arguments) {
            Ok(uttered) => uttered,
            Err(failure) => Uttered::silent(Err(failure)),
        }
    }

    fn answered<D: DispatchFiller>(
        self,
        run: &Run,
        arguments: impl Iterator<Item = OsString>,
    ) -> Result<Uttered, Failure>
    where
        O: Opens,
        H: FnOnce(PathBuf) -> D,
    {
        let request = Request::parsed(arguments)?;
        let located = request.located(self.environment)?;
        let store = run.port(request.opened(&located, &self.opened));
        let dispatch = run.port((self.agents.hired)(located.kept(Kept::Agents)));
        let clock = run.port(self.agents.clock);
        let shell = run.port(self.agents.shell);
        request.command().written(
            self.environment,
            Ground::new(run, self.files, located, &store),
            &Crew::new(&dispatch, &clock, &shell),
        )
    }
}

mod count;
mod failure;
pub(crate) mod outcome;
mod request;
pub(crate) mod uttered;

use std::ffi::OsString;

use console_ports::Environment;
use dispatch_ports::{ClockFiller, ShellFiller};
use file_ports::FilesFiller;
use port_core::Run;
use store_ports::Opens;

pub use count::Count;
pub use failure::Failure;
use request::{Crew, Ground, Kept, Request};
use uttered::{Settled, Uttered};

#[derive(Debug)]
pub struct Agents<C, G> {
    clock: C,
    shell: G,
}

#[derive(Debug)]
pub struct Cli<'environment, E: Environment, F: FilesFiller, O, C, G> {
    environment: &'environment E,
    files: F,
    opened: O,
    agents: Agents<C, G>,
}

impl<C: ClockFiller, G: ShellFiller> Agents<C, G> {
    pub fn new(clock: C, shell: G) -> Agents<C, G> {
        Agents { clock, shell }
    }
}

impl<'environment, E: Environment, F: FilesFiller + Clone, O: Opens, C: ClockFiller, G: ShellFiller>
    Cli<'environment, E, F, O, C, G>
{
    pub fn new(
        environment: &'environment E,
        files: F,
        opened: O,
        agents: Agents<C, G>,
    ) -> Cli<'environment, E, F, O, C, G> {
        Cli { environment, files, opened, agents }
    }

    pub fn rooted(self, arguments: impl Iterator<Item = OsString>) -> Uttered<Settled> {
        let run = Run::root();
        let uttered = self.report(&run, arguments);
        uttered.settled(run.finish())
    }

    pub fn report(self, run: &Run, arguments: impl Iterator<Item = OsString>) -> Uttered {
        match self.answered(run, arguments) {
            Ok(uttered) => uttered,
            Err(failure) => Uttered::silent(Err(failure)),
        }
    }

    fn answered(self, run: &Run, arguments: impl Iterator<Item = OsString>) -> Result<Uttered, Failure> {
        let request = Request::parsed(arguments)?;
        let located = request.located(self.environment)?;
        let store = run.port(request.opened(&located, &self.opened));
        let scratch = located.kept(Kept::Agents);
        let clock = run.port(self.agents.clock);
        let shell = run.port(self.agents.shell);
        request.command().written(
            self.environment,
            &Ground::new(run, run.port(self.files), located, &store),
            &Crew::new(&clock, &shell, scratch),
        )
    }
}

mod count;
mod failure;
pub(crate) mod outcome;
mod request;
pub(crate) mod uttered;

use std::ffi::OsString;

use console_ports::{Environment, Heard, Variables};
use dispatch_ports::{ClockFiller, ShellFiller, Wall};
use file_ports::Disk;
use port_core::Run;
use sqlite_ports::SqliteOpener;

pub use count::Count;
pub use failure::Failure;
pub use request::{Answer, Asked};
use request::{Crew, Ground, Kept, Located, Request, Tape};
use uttered::{Settled, Uttered};

#[derive(Debug)]
pub struct Agents<C, G> {
    clock: C,
    shell: G,
}

#[derive(Debug)]
pub struct Cli<'environment, E: Environment> {
    environment: &'environment E,
}

#[derive(Debug)]
struct Invoked {
    request: Request,
    located: Located,
    heard: Option<Heard>,
    variables: Variables,
}

impl<C: ClockFiller, G: ShellFiller> Agents<C, G> {
    pub fn new(clock: C, shell: G) -> Agents<C, G> {
        Agents { clock, shell }
    }
}

impl<'environment, E: Environment> Cli<'environment, E> {
    pub fn new(environment: &'environment E) -> Cli<'environment, E> {
        Cli { environment }
    }

    pub fn rooted<C: ClockFiller, G: ShellFiller>(
        &self,
        agents: Agents<C, G>,
        arguments: impl Iterator<Item = OsString>,
    ) -> Uttered<Settled> {
        let run = Run::root();
        let clock = run.port(agents.clock);
        let shell = run.port(agents.shell);
        let crew = Crew::new(&clock, &shell);
        let uttered = match self.invoked(arguments) {
            Ok(invoked) => invoked.replayed(&run, &crew, crew.now(), Vec::new()),
            Err(failure) => Uttered::silent(Err(failure)),
        };
        uttered.settled(run.finish().settled())
    }

    pub fn recorded(
        &self,
        arguments: impl Iterator<Item = OsString>,
        start: Wall,
        answers: &[Answer],
    ) -> Uttered<Settled> {
        let run = Run::root();
        let uttered = match self.invoked(arguments) {
            Ok(invoked) => invoked.passed(&run, start, answers),
            Err(failure) => Uttered::silent(Err(failure)),
        };
        uttered.settled(run.finish().settled())
    }

    fn invoked(&self, arguments: impl Iterator<Item = OsString>) -> Result<Invoked, Failure> {
        let request = Request::parsed(arguments)?;
        let variables = self.environment.variables();
        let located = request.located(&variables)?;
        let heard = request.command().heard(self.environment)?;
        Ok(Invoked { request, located, heard, variables })
    }
}

impl Invoked {
    fn replayed<C: ClockFiller, G: ShellFiller>(
        &self,
        run: &Run,
        crew: &Crew<'_, port_core::Port<'_, C>, port_core::Port<'_, G>>,
        start: Wall,
        answers: Vec<Answer>,
    ) -> Uttered {
        let uttered = self.passed(run, start, &answers);
        match AsRef::<[Asked]>::as_ref(&uttered).get(answers.len()) {
            Some(next) => {
                let answer = crew.performed(start, &answers, next);
                self.replayed(run, crew, start, [answers, vec![answer]].concat())
            }
            None => uttered,
        }
    }

    fn passed(&self, run: &Run, start: Wall, answers: &[Answer]) -> Uttered {
        let Invoked { request, located, heard, variables } = self;
        let tape = Tape::new(start, located.kept(Kept::Agents), answers);
        let store = run.port(request.opened(located, &SqliteOpener));
        let ground = Ground::new(run, Disk, located.clone(), &store, variables);
        let uttered = match request.command().written(heard.as_ref(), &ground, &tape) {
            Ok(uttered) => uttered,
            Err(failure) => Uttered::silent(Err(failure)),
        };
        uttered.asking(tape.asked())
    }
}

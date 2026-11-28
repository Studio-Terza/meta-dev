mod count;
mod failure;
pub(crate) mod outcome;
mod request;
pub(crate) mod uttered;

use std::ffi::OsString;

use console_ports::{Environment, Heard};
use dispatch_ports::{ClockFiller, ShellFiller, Wall};
use file_ports::Files;
use port_core::Run;
use store_ports::Opens;

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
pub struct Cli<'environment, E: Environment, F: Files, O> {
    environment: &'environment E,
    files: F,
    opened: O,
}

#[derive(Debug)]
struct Invoked {
    request: Request,
    located: Located,
    heard: Option<Heard>,
}

impl<C: ClockFiller, G: ShellFiller> Agents<C, G> {
    pub fn new(clock: C, shell: G) -> Agents<C, G> {
        Agents { clock, shell }
    }
}

impl<'environment, E: Environment, F: Files + Clone, O: Opens> Cli<'environment, E, F, O> {
    pub fn new(environment: &'environment E, files: F, opened: O) -> Cli<'environment, E, F, O> {
        Cli { environment, files, opened }
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
            Ok(invoked) => self.replayed(&run, &invoked, &crew, crew.now(), Vec::new()),
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
            Ok(invoked) => self.passed(&run, &invoked, start, answers),
            Err(failure) => Uttered::silent(Err(failure)),
        };
        uttered.settled(run.finish().settled())
    }

    fn invoked(&self, arguments: impl Iterator<Item = OsString>) -> Result<Invoked, Failure> {
        let request = Request::parsed(arguments)?;
        let located = request.located(self.environment)?;
        let heard = request.command().heard(self.environment)?;
        Ok(Invoked { request, located, heard })
    }

    fn replayed<C: ClockFiller, G: ShellFiller>(
        &self,
        run: &Run,
        invoked: &Invoked,
        crew: &Crew<'_, port_core::Port<'_, C>, port_core::Port<'_, G>>,
        start: Wall,
        answers: Vec<Answer>,
    ) -> Uttered {
        let uttered = self.passed(run, invoked, start, &answers);
        match AsRef::<[Asked]>::as_ref(&uttered).get(answers.len()) {
            Some(next) => {
                let answer = crew.performed(start, &answers, next);
                self.replayed(run, invoked, crew, start, [answers, vec![answer]].concat())
            }
            None => uttered,
        }
    }

    fn passed(&self, run: &Run, invoked: &Invoked, start: Wall, answers: &[Answer]) -> Uttered {
        let Invoked { request, located, heard } = invoked;
        let tape = Tape::new(start, located.kept(Kept::Agents), answers);
        let store = run.port(request.opened(located, &self.opened));
        let ground = Ground::new(run, self.files.clone(), located.clone(), &store);
        let uttered = match request.command().written(self.environment, heard.as_ref(), &ground, &tape) {
            Ok(uttered) => uttered,
            Err(failure) => Uttered::silent(Err(failure)),
        };
        uttered.asking(tape.asked())
    }
}

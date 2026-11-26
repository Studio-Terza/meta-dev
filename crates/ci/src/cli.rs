mod command;
mod ignored;
pub mod ports;

use std::ffi::OsString;

use console_ports::Teller;

pub use command::Unfolding;
use command::{Command, Failure, Telling};
use ports::Ports;

#[derive(Debug)]
pub struct Cli<'ports, P: Ports> {
    command: Result<Command, Failure<'static>>,
    ports: &'ports P,
}

impl<'ports, P: Ports> Cli<'ports, P> {
    pub fn parsed(arguments: impl Iterator<Item = OsString>, ports: &'ports P) -> Cli<'ports, P> {
        Cli { command: Command::parse(arguments), ports }
    }
}

impl<P: Ports> Teller for Cli<'_, P> {
    type Told<'teller>
        = Unfolding<'teller, P>
    where
        Self: 'teller;

    fn told(&self) -> Unfolding<'_, P> {
        Unfolding::new(self.telling(), self.ports)
    }
}

impl<P: Ports> Cli<'_, P> {
    fn telling(&self) -> Telling<'_> {
        match &self.command {
            Ok(command) => command.carry_out(self.ports),
            Err(refusal) => Telling::refused(refusal),
        }
    }
}

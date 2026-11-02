mod command;
pub mod ports;

use std::ffi::OsString;
use std::io::Write;
use std::process::ExitCode;

use command::Command;

use ports::Ports;

#[derive(Debug)]
pub struct Cli;

impl Cli {
    #[must_use]
    pub fn answer(
        arguments: impl Iterator<Item = OsString>,
        ports: &impl Ports,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> ExitCode {
        match Command::parse(arguments).and_then(|command| command.carry_out(ports, out)) {
            Ok(exit) => ExitCode::from(exit),
            Err(failure) => {
                writeln!(err, "{failure}").ok();
                ExitCode::FAILURE
            }
        }
    }
}

use crate::cli::command::Telling;
use console_ports::Shown;
use std::ffi::OsString;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct RatchetLower {
    record: gate_limits::Cli,
}

impl Step for RatchetLower {
    fn from_arguments(given: Vec<OsString>) -> Result<RatchetLower, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let record = arguments.taken()?;
        let measured = arguments.taken()?;
        arguments.ended()?;
        Ok(RatchetLower { record: gate_limits::Cli::from_args([record, measured].into_iter()) })
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let record = self.record.lower(ports.files())?;
        let said = vec![Shown::said(format_args!("record lowered to:\n{record}"))];
        Ok(Telling::reported(said, Ok(Exit::Success)))
    }
}

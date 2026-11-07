use std::ffi::OsString;
use std::io::Write;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct RatchetLower {
    record: gate_limits::Cli,
}

impl Step for RatchetLower {
    fn from_arguments(arguments: Vec<OsString>) -> Result<Self, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
        let record = arguments.taken()?;
        let measured = arguments.taken()?;
        arguments.ended()?;
        Ok(Self { record: gate_limits::Cli::from_args([record, measured].into_iter()) })
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let record = self.record.lower(ports.files())?;
        write!(out, "record lowered to:\n{record}")?;
        Ok(Exit::Success)
    }
}

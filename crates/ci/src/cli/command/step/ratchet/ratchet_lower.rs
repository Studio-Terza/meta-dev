use std::ffi::OsString;
use std::io::Write;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct RatchetLower {
    record: gate_limits::Cli,
}

impl Step for RatchetLower {
    fn from_arguments(arguments: Vec<OsString>) -> Result<RatchetLower, Failure<'static>> {
        match arguments.as_slice() {
            [_, _] => Ok(RatchetLower { record: gate_limits::Cli::from_args(arguments.into_iter()) }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let record = self.record.lower(ports.files())?;
        write!(out, "record lowered to:\n{record}")?;
        Ok(Exit::Success)
    }
}

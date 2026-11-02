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
    fn from_arguments(arguments: &[OsString]) -> Result<RatchetLower, Failure> {
        match arguments {
            [_, _] => Ok(RatchetLower { record: gate_limits::Cli::from_args(arguments.iter().cloned()) }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, _ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        let record = self.record.lower()?;
        write!(out, "record lowered to:\n{record}")?;
        Ok(Exit::Success)
    }
}

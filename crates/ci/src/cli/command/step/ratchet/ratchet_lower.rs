use crate::cli::command::Telling;
use console_ports::Shown;
use std::ffi::OsString;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Word};
use crate::cli::ports::{Exit, Ports};
use grammar_output::Record;

#[derive(Debug)]
pub struct RatchetLower {
    record: gate_limits::Cli,
}

#[derive(Debug, Record)]
#[record(source = Arguments)]
struct LoweredWords {
    #[field(Word)]
    record: OsString,
    #[field(Word)]
    measured: OsString,
}

impl From<LoweredWords> for RatchetLower {
    fn from(words: LoweredWords) -> RatchetLower {
        RatchetLower { record: gate_limits::Cli::from_args([words.record, words.measured].into_iter()) }
    }
}

impl Step for RatchetLower {
    fn from_arguments(given: Vec<OsString>) -> Result<RatchetLower, Failure<'static>> {
        LoweredWords::read(Arguments::from(given))?.ended().map(RatchetLower::from)
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let record = self.record.lower(ports.files())?;
        let said = vec![Shown::said(format_args!("record lowered to:\n{record}"))];
        Ok(Telling::reported(said, Ok(Exit::Success)))
    }
}

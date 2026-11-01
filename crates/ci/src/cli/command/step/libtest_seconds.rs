mod finished;

use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use finished::Finished;
use text_lines::Lines;

use super::Step;
use super::measured::Measured;
use super::seconds::Seconds;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct LibtestSeconds {
    measured: Measured,
    stage: String,
    output: PathBuf,
}

impl Step for LibtestSeconds {
    fn from_arguments(arguments: &[OsString]) -> Result<LibtestSeconds, Failure> {
        match arguments {
            [measured, stage, output] => Ok(LibtestSeconds {
                measured: Measured::file(Path::new(measured)),
                stage: stage.to_str().ok_or(Failure::Usage)?.to_owned(),
                output: PathBuf::from(output),
            }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, _ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure> {
        let text =
            fs::read_to_string(&self.output).map_err(|source| Failure::Io { path: self.output.clone(), source })?;
        let total: Seconds =
            Lines::from(text.as_str()).filter_map(|line| line.parse::<Finished>().ok()).map(Finished::seconds).sum();
        self.measured.append(&self.stage, total).map(|()| Exit::Success)
    }
}

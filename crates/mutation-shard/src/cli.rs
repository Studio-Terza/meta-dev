mod shard;

use std::ffi::OsString;

use console_ports::Answer;
use file_ports::Files;

use shard::Shard;

#[derive(Debug)]
pub struct Cli;

impl Cli {
    pub fn answer(arguments: impl Iterator<Item = OsString>, files: &impl Files) -> Answer {
        let given: Vec<OsString> = arguments.collect();
        Answer::reported(Shard::from_arguments(&given).and_then(|step| step.write_matrix(files)))
    }
}

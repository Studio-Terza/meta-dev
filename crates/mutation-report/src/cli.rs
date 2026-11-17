mod report;

use std::ffi::OsStr;

use console_ports::Answer;
use file_ports::Files;

use report::Report;

#[derive(Debug)]
pub struct Cli;

impl Cli {
    pub fn answer(arguments: &[impl AsRef<OsStr>], files: &impl Files) -> Answer {
        Answer::reported(Report::from_arguments(arguments).and_then(|report| report.write_summary(files)))
    }
}

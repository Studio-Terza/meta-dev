mod report;

use std::ffi::OsStr;
use std::io::Write;
use std::process::ExitCode;

use file_ports::Files;

use report::{Failure, Report};

#[derive(Debug)]
pub struct Cli;

impl Cli {
    pub fn answer(arguments: &[impl AsRef<OsStr>], files: &impl Files, err: &mut impl Write) -> ExitCode {
        Report::from_arguments(arguments)
            .and_then(|report| report.write_summary(files))
            .map_or_else(|failure| Self::refused(&failure, err), |()| ExitCode::SUCCESS)
    }

    fn refused(failure: &Failure<'_>, err: &mut impl Write) -> ExitCode {
        writeln!(err, "{failure}").ok();
        ExitCode::FAILURE
    }
}

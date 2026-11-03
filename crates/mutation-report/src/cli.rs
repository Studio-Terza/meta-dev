mod report;

use std::ffi::OsStr;
use std::io::Write;
use std::process::ExitCode;

use file_ports::Files;

use report::{Failure, Report};

#[derive(Debug)]
pub struct Cli;

impl Cli {
    #[must_use]
    pub fn answer(arguments: &[impl AsRef<OsStr>], files: &impl Files, err: &mut impl Write) -> ExitCode {
        match Report::from_arguments(arguments) {
            Ok(report) => Self::exited(report.write_summary(files), err),
            Err(failure) => Self::exited(Err(failure), err),
        }
    }

    fn exited(written: Result<(), Failure<'_>>, err: &mut impl Write) -> ExitCode {
        match written {
            Ok(()) => ExitCode::SUCCESS,
            Err(failure) => {
                writeln!(err, "{failure}").ok();
                ExitCode::FAILURE
            }
        }
    }
}

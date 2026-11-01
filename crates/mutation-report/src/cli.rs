mod report;

use std::ffi::OsString;
use std::io::Write;
use std::process::ExitCode;

use report::Report;

#[derive(Debug)]
pub struct Cli;

impl Cli {
    #[must_use]
    pub fn answer(arguments: impl Iterator<Item = OsString>, err: &mut impl Write) -> ExitCode {
        let arguments: Vec<OsString> = arguments.collect();
        match Report::from_arguments(&arguments).and_then(|step| step.write_summary()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(failure) => {
                writeln!(err, "{failure}").ok();
                ExitCode::FAILURE
            }
        }
    }
}

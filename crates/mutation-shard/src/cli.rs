mod shard;

use std::ffi::OsString;
use std::io::Write;
use std::process::ExitCode;

use shard::Shard;

#[derive(Debug)]
pub struct Cli;

impl Cli {
    #[must_use]
    pub fn answer(arguments: impl Iterator<Item = OsString>, err: &mut impl Write) -> ExitCode {
        let given: Vec<OsString> = arguments.collect();
        match Shard::from_arguments(&given).and_then(|step| step.write_matrix()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(failure) => {
                writeln!(err, "{failure}").ok();
                ExitCode::FAILURE
            }
        }
    }
}

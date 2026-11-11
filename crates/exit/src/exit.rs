use std::fmt::Display;
use std::io::Write;
use std::process::{ExitCode, Termination};

#[derive(Debug)]
pub struct Exit(ExitCode);

impl Exit {
    pub fn reported<Outcome, Failure: Display>(result: Result<Outcome, Failure>, err: &mut impl Write) -> Exit {
        result.map_or_else(|failure| Exit::failed(&failure, err), |_outcome| Exit(ExitCode::SUCCESS))
    }

    fn failed(failure: &impl Display, err: &mut impl Write) -> Exit {
        writeln!(err, "{failure}").ok();
        Exit(ExitCode::FAILURE)
    }
}

impl Termination for Exit {
    fn report(self) -> ExitCode {
        self.0
    }
}

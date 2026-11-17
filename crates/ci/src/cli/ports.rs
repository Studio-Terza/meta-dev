mod exit;
mod invocation;
mod run;
mod system;

use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::ExitCode;

use console_ports::{Complaint, Shown};
use file_ports::Files;

pub use exit::Exit;
pub use file_ports::Text;
pub use invocation::Invocation;
pub use run::Run;
pub use system::{System, Tee};

pub trait Ports {
    type Files: Files;

    fn files(&self) -> &Self::Files;

    fn run(&self, invocation: &Invocation) -> io::Result<Run>;

    fn output(&self, invocation: &Invocation) -> io::Result<Text>;

    fn logged(&self, invocation: &Invocation, log: &Path) -> io::Result<Run>;

    fn runs(&self, invocations: &[Invocation]) -> Vec<io::Result<Run>>;

    fn outputs(&self, invocations: &[Invocation]) -> Vec<io::Result<Text>>;

    fn reached(&self, arguments: &[impl AsRef<OsStr>]) -> Result<Text, Text> {
        let answered =
            reach::Cli::answer(arguments.iter().map(|argument| argument.as_ref().to_os_string()), self.files());
        let report: &Shown = answered.as_ref();
        let refusal: &Complaint = answered.as_ref();
        let text = |said: &str| Text::new(said.to_owned());
        Some(answered.code())
            .filter(|code| *code == ExitCode::SUCCESS)
            .map(|_| text(report.as_ref()))
            .ok_or_else(|| text(refusal.as_ref()))
    }

    fn reaches(&self, questions: &[Vec<impl AsRef<OsStr>>]) -> Vec<Result<Text, Text>> {
        questions.iter().map(|arguments| self.reached(arguments)).collect()
    }
}

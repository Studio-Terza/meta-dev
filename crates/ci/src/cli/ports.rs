mod exit;
mod invocation;
mod run;
mod system;
mod tee;

use std::io;
use std::path::Path;

use file_ports::Files;

pub use exit::Exit;
pub use file_ports::Text;
pub use invocation::Invocation;
pub use run::Run;
pub use system::System;
pub use tee::Tee;

pub trait Ports {
    type Files: Files;

    fn files(&self) -> &Self::Files;

    fn run(&self, invocation: &Invocation) -> io::Result<Run>;

    fn output(&self, invocation: &Invocation) -> io::Result<Text>;

    fn logged(&self, invocation: &Invocation, log: &Path) -> io::Result<Run>;

    fn runs(&self, invocations: &[Invocation]) -> Vec<io::Result<Run>>;

    fn outputs(&self, invocations: &[Invocation]) -> Vec<io::Result<Text>>;
}

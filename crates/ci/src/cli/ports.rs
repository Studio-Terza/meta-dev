mod exit;
mod invocation;
mod run;
mod system;
mod tee;

use std::io;
use std::path::Path;

pub use exit::Exit;
pub use invocation::Invocation;
pub use run::Run;
pub use system::System;
pub use tee::Tee;

pub trait Ports {
    fn run(&self, invocation: &Invocation) -> io::Result<Run>;

    fn output(&self, invocation: &Invocation) -> io::Result<String>;

    fn logged(&self, invocation: &Invocation, log: &Path) -> io::Result<Run>;
}

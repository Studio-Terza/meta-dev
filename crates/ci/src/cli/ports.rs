mod exit;
mod invocation;
mod run;
mod system;

use std::io;

pub use exit::Exit;
pub use invocation::Invocation;
pub use run::Run;
pub use system::System;

pub trait Ports {
    fn run(&self, invocation: &Invocation) -> io::Result<Run>;

    fn output(&self, invocation: &Invocation) -> io::Result<String>;
}

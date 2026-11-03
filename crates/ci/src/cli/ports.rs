mod exit;
mod invocation;
mod run;
mod system;
mod tee;
mod text;

use std::io;
use std::path::Path;

pub use exit::Exit;
pub use invocation::Invocation;
pub use run::Run;
pub use system::System;
pub use tee::Tee;
pub use text::Text;

pub trait Ports {
    fn run(&self, invocation: &Invocation) -> io::Result<Run>;

    fn output(&self, invocation: &Invocation) -> io::Result<Text>;

    fn logged(&self, invocation: &Invocation, log: &Path) -> io::Result<Run>;
}

mod clock;
mod command;
mod gh;
mod git;
#[cfg(test)]
mod hook_entry;
mod host;
mod jsonl;
mod log;

pub use clock::SystemClock;
pub use gh::GhCli;
pub use git::GitCli;
pub use host::{Exit, arguments, exit};
pub use log::JsonlLog;

use crate::Ports;

pub fn ports() -> Ports<JsonlLog, GitCli, GhCli, SystemClock> {
    Ports { log: JsonlLog::in_data_dir(), git: GitCli, github: GhCli, clock: SystemClock }
}

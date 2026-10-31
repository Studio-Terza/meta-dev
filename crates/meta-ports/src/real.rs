mod checkout;
mod clock;
mod command;
mod gh;
mod git;
mod git_env;
#[cfg(test)]
mod hook_entry;
mod host;
mod jsonl;
mod log;
#[cfg(test)]
mod stamp;
mod worker;

pub use clock::SystemClock;
pub use gh::GhCli;
pub use git::GitCli;
pub use host::{Exit, arguments, build, exit};
pub use log::JsonlLog;
pub use worker::HeadlessSessions;

use crate::Ports;

pub fn ports() -> Ports<JsonlLog, GitCli, GhCli, SystemClock, HeadlessSessions> {
    Ports {
        log: JsonlLog::in_data_dir(),
        git: GitCli,
        github: GhCli,
        clock: SystemClock,
        sessions: HeadlessSessions,
        build: build(),
    }
}

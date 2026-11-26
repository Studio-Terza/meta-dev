#![deny(clippy::wildcard_enum_match_arm)]

mod cli;
mod hook_event;

pub use cli::outcome::Outcome;
pub use cli::uttered::Uttered;
pub use cli::{Agents, Cli, Count, Failure};
pub use hook_event::HookEvent;

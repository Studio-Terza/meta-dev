#![deny(clippy::wildcard_enum_match_arm)]

mod cli;
mod hook_event;

pub(crate) use cli::outcome::Outcome;
pub use cli::{Agents, Cli};
pub(crate) use hook_event::HookEvent;

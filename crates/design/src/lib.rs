#![deny(clippy::wildcard_enum_match_arm)]

mod arguments;
mod cli;
mod hook_event;

pub use arguments::Arguments;
pub(crate) use cli::outcome::Outcome;
pub use cli::{Agents, Cli};
pub(crate) use hook_event::HookEvent;

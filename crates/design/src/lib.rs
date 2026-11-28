#![deny(clippy::wildcard_enum_match_arm)]

mod cli;
mod hook_event;

use cli::outcome::Outcome;
pub use cli::{Agents, Cli};
use hook_event::HookEvent;

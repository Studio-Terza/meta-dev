#![deny(clippy::wildcard_enum_match_arm)]

mod cli;
mod hook_event;
mod lint;

use cli::outcome::Outcome;
pub use cli::{Agents, Cli, Failure};
use hook_event::HookEvent;
pub use lint::{Lint, Verdict};

#![deny(clippy::wildcard_enum_match_arm)]

mod cli;
mod hook_event;

pub use cli::noted::{Miss, Noted, Reconciliation, Reference};
pub use cli::outcome::Outcome;
pub use cli::output::boundary::RefusedLink;
pub use cli::output::{Anchored, Attempted, Count, Flailing, Lapsed, Line, Remark, Summary};
pub use cli::uttered::Uttered;
pub use cli::{Agents, Cli, Failure, Flail, Flails, Sequel, Unmatched, Unresolved, Viewed};
pub use hook_event::HookEvent;

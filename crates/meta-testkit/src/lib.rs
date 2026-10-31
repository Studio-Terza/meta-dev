//! Test-only helpers, used only as a dev-dependency. Production code never depends on this crate.

mod command_line;
mod scratch;

pub use command_line::words;
pub use scratch::Scratch;

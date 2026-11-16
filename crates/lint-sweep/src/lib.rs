mod scoped;
mod sweep;

pub use lint_core::{Address, Descend, Facts, Kind, Watch, Watching};
pub use scoped::{Attaches, Attachment, Scoped};
pub use sweep::{Finds, Gathers, Member, Sorts, Sweep, Unwatched, Watchers};

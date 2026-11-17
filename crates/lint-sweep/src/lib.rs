mod scoped;
mod sweep;

pub use lint_core::{Address, Descend, Facts, Fold, Folding, Kind, Visit, Watch, Watching};
pub use scoped::{Attaches, Attachment, Scoped};
pub use sweep::{Finds, Gathers, Member, Sweep, Watchers};

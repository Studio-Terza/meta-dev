mod scoped;
mod sweep;

pub use lint_core::{Address, Descend, Facts, Fold, Folding, Kind, Visit, Watching};
pub use scoped::{Attaching, AttachingGroups, Attachment, Grouped, Scoped};
pub use sweep::{Finds, Gathers, Member, Sweep, Watchers};

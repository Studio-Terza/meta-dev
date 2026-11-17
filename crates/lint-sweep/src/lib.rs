mod scoped;
mod sweep;

pub use lint_core::{
    Address, Around, Contribute, Contributed, Contributing, Contribution, Descend, Facts, Kind, Scope, Watch, Watching,
};
pub use scoped::{Attaches, Attachment, Scoped};
pub use sweep::{Concludes, Finds, Gathers, Member, Sorts, Sweep, Watchers};

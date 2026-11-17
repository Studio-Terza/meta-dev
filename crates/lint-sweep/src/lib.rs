mod attaching;
mod drawn;
#[macro_use]
mod hooks;
mod scoped;
mod sweep;

pub use attaching::{Attaching, AttachingGroups, Grouped};
pub use drawn::Drawn;
pub use hooks::{Fold, Folding, Kind, Visit};
pub use lint_core::{Address, Descend, Facts};
pub use scoped::{Attachment, Scoped};
pub use sweep::{Finds, Gathers, Member, Sweep, Watchers};

pub type Watching<'run> = dyn for<'ast> hooks::Watch<'ast> + 'run;

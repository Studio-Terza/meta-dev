mod index;
mod inside_sibling;
mod target;

pub use index::ModuleIndex;
pub use inside_sibling::{InsideSibling, Siding};
pub use target::Target;

use lint_layout::InFile;
use lint_placement::Nest;

pub type ModulePath<'file> = InFile<'file, Nest<'file>>;

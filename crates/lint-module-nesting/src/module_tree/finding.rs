use std::fmt;

use super::module_path::{ModulePath, Target};
use super::reexport_count::ReexportCount;
use lint_core::Fix;
use lint_core::Most;

pub enum Finding<'file, 'path> {
    UsedFromOneSibling { home: ModulePath<'file> },
    ReachesPastModRs { target: &'path Target<'path> },
    CrowdedReexports { count: ReexportCount, most: Most },
}

impl fmt::Display for Finding<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::UsedFromOneSibling { home } => write!(formatter, "used only from {home}"),
            Finding::ReachesPastModRs { target } => write!(formatter, "reaches into {target}"),
            Finding::CrowdedReexports { count, most } => {
                write!(formatter, "re-exports {count} items, at most {most} allowed")
            }
        }
    }
}

impl lint_core::Finding for Finding<'_, '_> {
    fn fix(&self) -> Fix {
        match self {
            Finding::UsedFromOneSibling { .. } => Fix::new("move it inside that module"),
            Finding::ReachesPastModRs { .. } => Fix::new("import the parent's re-export instead"),
            Finding::CrowdedReexports { .. } => Fix::new("split the module"),
        }
    }
}

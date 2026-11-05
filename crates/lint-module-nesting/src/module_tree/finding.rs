use std::fmt;

use super::module_path::Target;
use super::reexport_count::ReexportCount;
use lint_core::Bound;
use lint_core::Fix;

pub enum Finding<'path> {
    ReachesPastModRs { target: &'path Target<'path> },
    CrowdedReexports { count: ReexportCount, most: Bound },
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::ReachesPastModRs { target } => write!(formatter, "reaches into {target}"),
            Finding::CrowdedReexports { count, most } => {
                write!(formatter, "re-exports {count} items, at most {most} allowed")
            }
        }
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        match self {
            Finding::ReachesPastModRs { .. } => Fix::new("import the parent's re-export instead"),
            Finding::CrowdedReexports { .. } => Fix::new("split the module"),
        }
    }
}

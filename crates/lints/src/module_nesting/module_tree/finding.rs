use std::fmt;

use super::module_path::ModulePath;
use super::reexport_count::ReexportCount;
use lint_core::Most;

pub enum Finding {
    UsedFromOneSibling { home: ModulePath },
    ReachesPastModRs { target: ModulePath },
    CrowdedReexports { count: ReexportCount, most: Most },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::UsedFromOneSibling { home } => {
                write!(formatter, "used only from {home}; move it inside that module")
            }
            Finding::ReachesPastModRs { target } => {
                write!(formatter, "reaches into {target}; import the parent's re-export instead")
            }
            Finding::CrowdedReexports { count, most } => {
                write!(formatter, "re-exports {count} items, at most {most} allowed; split the module")
            }
        }
    }
}

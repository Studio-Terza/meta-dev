use std::fmt;

use super::module_path::ModulePath;
use super::reexport_count::ReexportCount;
use lint_core::Fix;
use lint_core::Most;

pub enum Finding<'file> {
    UsedFromOneSibling { home: ModulePath<'file> },
    ReachesPastModRs { target: ModulePath<'file> },
    CrowdedReexports { count: ReexportCount, most: Most },
}

impl fmt::Display for Finding<'_> {
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

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new(match self {
            Finding::UsedFromOneSibling { .. } => "move it inside that module",
            Finding::ReachesPastModRs { .. } => "import the parent's re-export instead",
            Finding::CrowdedReexports { .. } => "split the module",
        })
    }
}

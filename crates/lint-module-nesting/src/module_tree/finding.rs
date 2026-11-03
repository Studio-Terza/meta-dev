use std::fmt;

use super::module_path::{ModulePath, Target};
use lint_core::Fix;

pub enum Finding<'file, 'path> {
    UsedFromOneSibling { home: ModulePath<'file> },
    ReachesPastModRs { target: &'path Target<'path> },
}

impl fmt::Display for Finding<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::UsedFromOneSibling { home } => write!(formatter, "used only from {home}"),
            Finding::ReachesPastModRs { target } => write!(formatter, "reaches into {target}"),
        }
    }
}

impl lint_core::Finding for Finding<'_, '_> {
    fn fix(&self) -> Fix {
        Fix::new(match self {
            Finding::UsedFromOneSibling { .. } => "move it inside that module",
            Finding::ReachesPastModRs { .. } => "import the parent's re-export instead",
        })
    }
}

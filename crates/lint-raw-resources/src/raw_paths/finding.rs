use std::fmt;

use lint_core::Fix;
use code_model::ItemPath;
use lint_layout::UsePath;

pub enum Finding<'path> {
    OutsidePorts(&'path ItemPath),
    OutsideGuard(&'path ItemPath),
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::OutsidePorts(path) => write!(formatter, "calls {}", Self::shown(path)),
            Finding::OutsideGuard(path) => write!(formatter, "calls {} outside a guard", Self::shown(path)),
        }
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        match self {
            Finding::OutsidePorts(_) => {
                Fix::new("put it in a guard behind a port: a crate whose name ends in ports, or a module named ports")
            }
            Finding::OutsideGuard(_) => Fix::new("in a port, put it in an impl of a type that implements Drop"),
        }
    }
}

impl Finding<'_> {
    fn shown(path: &ItemPath) -> UsePath {
        path.as_ref().iter().cloned().collect()
    }
}

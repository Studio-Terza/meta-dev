use std::fmt;

use lint_core::Fix;
use lint_layout::UsePath;

pub enum Finding<'path> {
    OutsidePorts(&'path UsePath),
    OutsideGuard(&'path UsePath),
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::OutsidePorts(path) => write!(formatter, "calls {path}"),
            Finding::OutsideGuard(path) => write!(formatter, "calls {path} outside a guard"),
        }
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new(match self {
            Finding::OutsidePorts(_) => {
                "put it in a guard behind a port: a crate whose name ends in ports, or a module named ports"
            }
            Finding::OutsideGuard(_) => "in a port, put it in an impl of a type that implements Drop",
        })
    }
}

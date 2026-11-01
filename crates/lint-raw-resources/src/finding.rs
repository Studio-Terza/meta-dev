use std::fmt;

use lint_layout::UsePath;

pub enum Finding {
    OutsidePorts(UsePath),
    OutsideGuard(UsePath),
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::OutsidePorts(path) => write!(
                formatter,
                "calls {path}; put it in a guard behind a port: a crate whose name ends in ports, or a module named ports"
            ),
            Finding::OutsideGuard(path) => write!(
                formatter,
                "calls {path} outside a guard; in a port, put it in an impl of a type that implements Drop"
            ),
        }
    }
}

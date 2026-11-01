use std::fmt;

use lint_layout::UsePath;

pub struct Finding(UsePath);

impl From<UsePath> for Finding {
    fn from(path: UsePath) -> Finding {
        Finding(path)
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "calls {}; put it in a guard behind a port: a crate whose name ends in ports, or a module named ports",
            self.0
        )
    }
}

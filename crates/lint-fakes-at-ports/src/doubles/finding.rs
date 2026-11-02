use std::fmt;

use lint_layout::{Name, UsePath};

#[derive(Debug)]
pub enum Finding {
    Unported(Name),
    Beside(Name, UsePath),
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::Unported(name) => write!(
                formatter,
                "{name} is named as a test double but implements no port; implement a trait declared in a ports crate or module, or rename it"
            ),
            Finding::Beside(name, path) => write!(
                formatter,
                "{name} implements {path}, which is neither a port nor a std trait; a test double implements only traits declared in a ports crate or module, besides derives and std traits"
            ),
        }
    }
}

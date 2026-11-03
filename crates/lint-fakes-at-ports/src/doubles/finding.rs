use std::fmt;

use lint_layout::{Fix, Name, UsePath};

#[derive(Debug)]
pub enum Finding<'double> {
    Unported(&'double Name),
    Beside(&'double Name, UsePath),
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::Unported(name) => write!(formatter, "{name} is named as a test double but implements no port"),
            Finding::Beside(name, path) => {
                write!(formatter, "{name} implements {path}, which is neither a port nor a std trait")
            }
        }
    }
}

impl lint_layout::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new(match self {
            Finding::Unported(_) => "implement a trait declared in a ports crate or module, or rename it",
            Finding::Beside(..) => {
                "a test double implements only traits declared in a ports crate or module, besides derives and std traits"
            }
        })
    }
}

use std::fmt;

use lint_core::Fix;
use lint_layout::UsePath;

#[derive(Debug)]
pub struct Finding<'path>(&'path UsePath);

impl<'path> From<&'path UsePath> for Finding<'path> {
    fn from(path: &'path UsePath) -> Self {
        Finding(path)
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "calls {} in test code", self.0)
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        Fix::new("use a fake of the port or a library API such as golden!")
    }
}

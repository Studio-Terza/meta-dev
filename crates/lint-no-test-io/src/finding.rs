use std::fmt;

use lint_layout::UsePath;

#[derive(Debug)]
pub struct Finding<'path>(&'path UsePath);

impl<'path> From<&'path UsePath> for Finding<'path> {
    fn from(path: &'path UsePath) -> Finding<'path> {
        Finding(path)
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "calls {} in test code; use a fake of the port or a library API such as golden!, or move the test into a test target or module named ports",
            self.0
        )
    }
}

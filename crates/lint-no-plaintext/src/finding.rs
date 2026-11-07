use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

use crate::kinds::Reading;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Finding<'model> {
    reading: Reading,
    context: &'model Name,
}

impl<'model> Finding<'model> {
    pub(crate) fn new(reading: Reading, context: &'model Name) -> Finding<'model> {
        Finding { reading, context }
    }
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        match self.reading {
            Reading::Plain => Fix::new(
                "split it into typed parts, or use a newtype that validates (derive newtype::CheckedDeserialize, which reads through its impl of newtype::Checked)",
            ),
            Reading::Unresolved => Fix::new(
                "import it explicitly, by a path or a named use rather than a glob, so the lint can tell whether it reads the type from a file",
            ),
        }
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reading {
            Reading::Plain => write!(formatter, "free text in {}", self.context),
            Reading::Unresolved => write!(formatter, "derive not resolvable beside the text in {}", self.context),
        }
    }
}

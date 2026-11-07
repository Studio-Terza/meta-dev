use std::fmt;

use lint_core::Fix;
use lint_layout::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Finding<'model> {
    Plain(&'model Name),
    UnresolvedDerive(&'model Name),
    UnresolvedType(&'model Name),
}

impl lint_core::Finding for Finding<'_> {
    fn fix(&self) -> Fix {
        match self {
            Finding::Plain(_) => Fix::new(
                "split it into typed parts, or use a newtype that validates (derive newtype::CheckedDeserialize, which reads through its impl of newtype::Checked)",
            ),
            Finding::UnresolvedDerive(_) => Fix::new(
                "import it explicitly, by a path or a named use rather than a glob, so the lint can tell whether it reads the type from a file",
            ),
            Finding::UnresolvedType(_) => Fix::new(
                "import it explicitly, by a path or a named use rather than a glob, so the lint can tell whether the field holds text",
            ),
        }
    }
}

impl fmt::Display for Finding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::Plain(context) => write!(formatter, "free text in {context}"),
            Finding::UnresolvedDerive(subject) => write!(formatter, "derive not resolvable on {subject}"),
            Finding::UnresolvedType(field) => write!(formatter, "type not resolvable in {field}"),
        }
    }
}

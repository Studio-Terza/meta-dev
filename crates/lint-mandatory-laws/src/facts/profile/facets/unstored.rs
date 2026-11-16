use std::fmt;

use crate::subject::Subject;
use lint_core::Finding;
use lint_core::Fix;

#[derive(Debug)]
pub(crate) struct Unstored<'facts, 'file> {
    subject: &'facts Subject<'file>,
}

impl<'facts, 'file> From<&'facts Subject<'file>> for Unstored<'facts, 'file> {
    fn from(subject: &'facts Subject<'file>) -> Unstored<'facts, 'file> {
        Unstored { subject }
    }
}

impl fmt::Display for Unstored<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "`{}` is a stored record without create, read, update and delete laws",
            AsRef::<lint_layout::Name>::as_ref(self.subject)
        )
    }
}

impl Finding for Unstored<'_, '_> {
    fn fix(&self) -> Fix {
        Fix::new("add a `laws::crud!` in a test of the same package that names the type")
    }
}

use std::fmt;

use super::Pair;
use crate::subject::Subject;
use lint_core::Finding;
use lint_core::Fix;

#[derive(Debug)]
pub(crate) struct Obligation<'facts, 'file> {
    subject: &'facts Subject<'file>,
    pair: Pair<'facts>,
}

impl<'facts, 'file> Obligation<'facts, 'file> {
    pub(crate) fn new(subject: &'facts Subject<'file>, pair: Pair<'facts>) -> Obligation<'facts, 'file> {
        Obligation { subject, pair }
    }
}

impl fmt::Display for Obligation<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "`{}` has the conversion pair {} (Display and FromStr, Serialize and Deserialize, encode and decode, or to_x and from_x)",
            AsRef::<lint_layout::Name>::as_ref(self.subject),
            self.pair
        )
    }
}

impl Finding for Obligation<'_, '_> {
    fn fix(&self) -> Fix {
        Fix::new("add a `laws::roundtrip!` in a test of the same package that names the type and the pair")
    }
}

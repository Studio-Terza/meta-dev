use std::collections::BTreeSet;

use crate::subject::Subject;
use law_facts::{Kind, Law, Laws};
use lint_layout::Name;

#[derive(Debug, Default)]
pub(crate) struct Coverage<'file> {
    laws: Laws<'file>,
}

impl<'file> From<Laws<'file>> for Coverage<'file> {
    fn from(laws: Laws<'file>) -> Coverage<'file> {
        Coverage { laws }
    }
}

impl Coverage<'_> {
    pub(crate) fn roundtrips_naming(&self, subject: &Subject<'_>) -> Vec<&BTreeSet<Name>> {
        self.laws
            .naming(subject.package(), &[subject.owner()])
            .into_iter()
            .filter(|law| law.kind() == Kind::Roundtrip)
            .map(Law::mentions)
            .collect()
    }
}

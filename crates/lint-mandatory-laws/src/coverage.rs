use std::collections::BTreeSet;

use crate::subject::Subject;
use law_facts::{Kind, Law, Laws};
use lint_core::SourceFile;
use lint_layout::Name;

#[derive(Debug, Default)]
pub(crate) struct Coverage<'file> {
    laws: Laws<'file>,
}

impl<'file> Coverage<'file> {
    pub(crate) fn read_all(files: &'file [SourceFile]) -> Coverage<'file> {
        Coverage { laws: Laws::from(files) }
    }

    pub(crate) fn roundtrips_naming(&self, subject: &Subject<'_>) -> Vec<&BTreeSet<Name>> {
        self.laws
            .naming(subject.package(), &[subject.owner()])
            .into_iter()
            .filter(|law| law.kind() == Kind::Roundtrip)
            .map(Law::mentions)
            .collect()
    }
}

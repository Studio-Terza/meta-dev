mod kind;
mod law;
mod reader;

use std::collections::BTreeSet;

use crate::subject::Subject;
use kind::Kind;
use law::Law;
use lint_core::SourceFile;
use lint_layout::Name;
use reader::Reader;

#[derive(Debug, Default)]
pub(crate) struct Coverage<'file> {
    laws: Vec<Law<'file>>,
}

impl<'file> Coverage<'file> {
    pub(crate) fn read_all(files: &'file [SourceFile]) -> Coverage<'file> {
        files.iter().fold(Coverage::default(), Coverage::read)
    }

    pub(crate) fn law_naming(&self, subject: &Subject<'_>) -> Option<&Law<'file>> {
        self.laws.iter().find_map(|law| law.naming(subject))
    }

    pub(crate) fn roundtrips_naming(&self, subject: &Subject<'_>) -> Vec<&BTreeSet<Name>> {
        self.laws
            .iter()
            .filter_map(|law| law.naming(subject))
            .filter(|law| law.kind() == Kind::Roundtrip)
            .map(Law::mentions)
            .collect()
    }

    fn read(mut self, file: &'file SourceFile) -> Coverage<'file> {
        self.laws.extend(Reader::read(file));
        file.test_modules().iter().fold(self, Coverage::read)
    }
}

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
pub(crate) struct Coverage {
    laws: Vec<Law>,
}

impl Coverage {
    pub(crate) fn read_all(files: &[SourceFile]) -> Coverage {
        files.iter().fold(Coverage::default(), Coverage::read)
    }

    pub(crate) fn names(&self, subject: &Subject) -> bool {
        self.laws.iter().any(|law| law.names(subject))
    }

    pub(crate) fn roundtrips_naming(&self, subject: &Subject) -> Vec<&BTreeSet<Name>> {
        self.laws.iter().filter(|law| law.kind() == Kind::Roundtrip && law.names(subject)).map(Law::mentions).collect()
    }

    fn read(mut self, file: &SourceFile) -> Coverage {
        self.laws.extend(Reader::read(file));
        file.test_modules().iter().fold(self, Coverage::read)
    }
}

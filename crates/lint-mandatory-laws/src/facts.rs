mod exposure;
mod profile;
mod reader;

use std::collections::BTreeMap;

use crate::coverage::Coverage;
use crate::subject::Subject;
use exposure::Exposure;
use lint_core::{SourceFile, Violation};
use profile::{Facets, Profile};
use reader::Reader;

#[derive(Debug, Default)]
pub(crate) struct Facts<'file> {
    profiles: BTreeMap<Subject<'file>, Facets<'file>>,
}

impl<'file> Facts<'file> {
    pub(crate) fn read_all(files: &'file [SourceFile]) -> Self {
        files.iter().filter_map(SourceFile::shipped).fold(Facts::default(), Facts::read)
    }

    pub(crate) fn unmet(&self, coverage: &Coverage<'_>) -> Vec<Violation<'file>> {
        self.profiles
            .iter()
            .filter(|(_, facets)| facets.exposure() == Exposure::Public)
            .flat_map(|(subject, facets)| facets.unmet(subject, coverage))
            .collect()
    }

    fn read(self, file: &'file SourceFile) -> Self {
        Reader::read(file).into_iter().fold(self, Facts::added)
    }

    fn added(mut self, profile: Profile<'file>) -> Self {
        profile.filed_in(&mut self.profiles);
        self
    }
}

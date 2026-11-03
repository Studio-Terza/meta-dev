mod exposure;
mod need;
mod obligation;
mod pair;
mod profile;
mod reader;

use std::collections::BTreeMap;

use crate::coverage::Coverage;
use crate::subject::Subject;
use exposure::Exposure;
use lint_core::SourceFile;
use obligation::Obligation;
use profile::Profile;
use reader::Reader;

#[derive(Debug, Default)]
pub(crate) struct Facts<'file> {
    profiles: BTreeMap<Subject, Profile<'file>>,
}

impl<'file> Facts<'file> {
    pub(crate) fn read_all(files: &'file [SourceFile]) -> Facts<'file> {
        files.iter().filter_map(SourceFile::shipped).fold(Facts::default(), Facts::read)
    }

    pub(crate) fn unmet(&self, coverage: &Coverage) -> Vec<Obligation<'file>> {
        self.profiles
            .values()
            .filter(|profile| profile.exposure() == Exposure::Public)
            .flat_map(|profile| profile.unmet(coverage))
            .collect()
    }

    fn read(self, file: &'file SourceFile) -> Facts<'file> {
        Reader::read(file).into_iter().fold(self, Facts::added)
    }

    fn added(mut self, profile: Profile<'file>) -> Facts<'file> {
        let subject = profile.subject().clone();
        let merged = match self.profiles.remove(&subject) {
            Some(earlier) => earlier.merged(profile),
            None => profile,
        };
        self.profiles.insert(subject, merged);
        self
    }
}

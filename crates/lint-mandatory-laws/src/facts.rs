mod profile;
mod reader;

use std::collections::BTreeMap;

use crate::coverage::Coverage;
use crate::obligation::Obligation;
use crate::subject::Subject;
use lint_core::SourceFile;
use profile::Profile;
use reader::Reader;

#[derive(Debug, Default)]
pub(crate) struct Facts {
    profiles: BTreeMap<Subject, Profile>,
}

impl Facts {
    pub(crate) fn read_all(files: &[SourceFile]) -> Facts {
        files.iter().filter(|file| file.is_shipped()).fold(Facts::default(), Facts::read)
    }

    pub(crate) fn unmet(&self, coverage: &Coverage) -> Vec<Obligation> {
        self.profiles.values().flat_map(|profile| profile.unmet(coverage)).collect()
    }

    fn read(mut self, file: &SourceFile) -> Facts {
        Reader::read(file).into_iter().for_each(|profile| self.add(profile));
        self
    }

    fn add(&mut self, profile: Profile) {
        let subject = profile.subject().clone();
        let merged = match self.profiles.remove(&subject) {
            Some(earlier) => earlier.merged(profile),
            None => profile,
        };
        self.profiles.insert(subject, merged);
    }
}

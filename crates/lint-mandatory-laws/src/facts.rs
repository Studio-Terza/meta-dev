mod exposure;
mod profile;
mod reader;

use std::collections::BTreeMap;

use crate::coverage::Coverage;
use crate::subject::Subject;
use exposure::Exposure;
use lint_core::{Parsed, Violation};
use profile::{Facets, Profile};
use reader::Reader;

#[derive(Debug, Default)]
pub struct Facts<'file> {
    profiles: BTreeMap<Subject<'file>, Facets<'file>>,
}

impl<'file> FromIterator<Profiles<'file>> for Facts<'file> {
    fn from_iter<Parts: IntoIterator<Item = Profiles<'file>>>(parts: Parts) -> Facts<'file> {
        parts.into_iter().flat_map(|part| part.0).fold(Facts::default(), Facts::added)
    }
}

impl<'file> Facts<'file> {
    pub(crate) fn unmet(&self, coverage: &Coverage<'_>) -> Vec<Violation<'file>> {
        self.profiles
            .iter()
            .filter(|(_, facets)| facets.exposure() == Exposure::Public)
            .flat_map(|(subject, facets)| facets.unmet(subject, coverage))
            .collect()
    }

    fn added(mut self, profile: Profile<'file>) -> Facts<'file> {
        profile.filed_in(&mut self.profiles);
        self
    }
}

#[derive(Debug)]
pub struct Profiles<'file>(Vec<Profile<'file>>);

impl<'file> From<Parsed<'_, 'file>> for Profiles<'file> {
    fn from(parsed: Parsed<'_, 'file>) -> Profiles<'file> {
        Profiles(parsed.file().shipped().map_or_default(|_| Reader::read(parsed)))
    }
}

mod facets;

use std::collections::BTreeMap;

use super::exposure::Exposure;
use crate::subject::Subject;
use lint_core::Site;
use lint_layout::Name;

pub(crate) use facets::Facets;

#[derive(Debug)]
pub(crate) struct Profile<'file> {
    subject: Subject<'file>,
    facets: Facets<'file>,
}

impl<'file> Profile<'file> {
    pub(crate) fn declared(
        subject: Subject<'file>,
        site: Site<'file>,
        exposure: Exposure,
        facts: Vec<Name>,
    ) -> Profile<'file> {
        Profile { subject, facets: Facets::new(site, exposure, facts) }
    }

    pub(crate) fn filed_in(self, profiles: &mut BTreeMap<Subject<'file>, Facets<'file>>) {
        let merged = self.facets.after(profiles.remove(&self.subject));
        profiles.insert(self.subject, merged);
    }
}

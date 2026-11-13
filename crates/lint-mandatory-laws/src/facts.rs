mod exposure;
mod profile;
mod reading;

use std::collections::BTreeMap;

use graph::Model;
use lint_core::Violation;

use crate::coverage::Coverage;
use crate::subject::Subject;
use exposure::Exposure;
use profile::{Facets, Profile};
use reading::Reading;

#[derive(Debug, Default)]
pub struct Facts<'model> {
    profiles: BTreeMap<Subject<'model>, Facets<'model>>,
}

impl<'model> From<&'model Model> for Facts<'model> {
    fn from(model: &'model Model) -> Facts<'model> {
        Reading::from(model).profiles().into_iter().fold(Facts::default(), Facts::added)
    }
}

impl<'model> Facts<'model> {
    pub(crate) fn unmet(&self, coverage: &Coverage<'_>) -> Vec<Violation<'model>> {
        self.profiles
            .iter()
            .filter(|(_, facets)| facets.exposure() == Exposure::Public)
            .flat_map(|(subject, facets)| facets.unmet(subject, coverage))
            .collect()
    }

    fn added(mut self, profile: Profile<'model>) -> Facts<'model> {
        profile.filed_in(&mut self.profiles);
        self
    }
}

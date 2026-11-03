use std::collections::BTreeSet;

use crate::scopes::Scopes;
use lint_layout::{Name, Word};

#[derive(Debug)]
pub(crate) struct Wanted(Option<BTreeSet<Name>>);

impl Wanted {
    pub(crate) const EVERY: Wanted = Wanted(None);

    pub(crate) fn among(scopes: &Scopes, finals: &[Word<'_>]) -> Wanted {
        let renamed = scopes.renamed_to(finals);
        Wanted(Some(finals.iter().map(|name| Name::from(*name)).chain(renamed).collect()))
    }

    pub(crate) fn holds<'name>(&self, name: &'name Name) -> Option<&'name Name> {
        match &self.0 {
            None => Some(name),
            Some(names) => names.contains(name).then_some(name),
        }
    }
}

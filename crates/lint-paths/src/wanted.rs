use std::collections::BTreeSet;

use crate::scopes::Scopes;
use lint_layout::{Name, Word};

#[derive(Debug)]
pub(crate) struct Wanted(BTreeSet<Name>);

impl Wanted {
    pub(crate) fn among(scopes: &Scopes, finals: &[Word<'_>]) -> Wanted {
        let renamed = scopes.renamed_to(finals);
        Wanted(finals.iter().map(|name| Name::from(*name)).chain(renamed).collect())
    }

    pub(crate) fn holds(&self, name: &Name) -> bool {
        self.0.contains(name)
    }
}

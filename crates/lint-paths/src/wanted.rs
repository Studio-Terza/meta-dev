use std::collections::BTreeSet;

use crate::scopes::Scopes;
use lint_layout::{Name, Word};

#[derive(Debug)]
pub(crate) enum Wanted<'run> {
    Every,
    Among { finals: &'run [Word<'run>], renamed: BTreeSet<&'run Name> },
}

impl<'run> Wanted<'run> {
    pub(crate) fn among(scopes: &'run Scopes<'_>, finals: &'run [Word<'run>]) -> Wanted<'run> {
        Wanted::Among { finals, renamed: scopes.renamed_to(finals).collect() }
    }

    pub(crate) fn holds<'name>(&self, name: &'name Name) -> Option<&'name Name> {
        match self {
            Wanted::Every => Some(name),
            Wanted::Among { finals, renamed } => {
                (finals.iter().any(|word| name == word) || renamed.contains(name)).then_some(name)
            }
        }
    }
}

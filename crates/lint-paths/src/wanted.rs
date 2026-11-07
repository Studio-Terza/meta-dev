use std::collections::BTreeSet;

use crate::scopes::Scopes;
use lint_layout::{Name, Word};

#[derive(Debug)]
pub(crate) enum Wanted<'run> {
    Every,
    Among { finals: &'run [Word<'run>], renamed: BTreeSet<&'run Name> },
}

impl<'run> Wanted<'run> {
    pub(crate) fn among(scopes: &'run Scopes<'_>, finals: &'run [Word<'run>]) -> Self {
        Wanted::Among { finals, renamed: scopes.renamed_to(finals).collect() }
    }

    pub(crate) fn holds<'name>(&self, name: &'name Name) -> Option<&'name Name> {
        match self {
            Wanted::Every => Some(name),
            Wanted::Among { finals, renamed } => Self::among_named(finals, renamed, name),
        }
    }

    fn among_named<'name>(finals: &[Word<'_>], renamed: &BTreeSet<&Name>, name: &'name Name) -> Option<&'name Name> {
        finals.iter().find(|word| *name == Name::from(**word)).map(|_| name).or_else(|| renamed.get(name).map(|_| name))
    }
}

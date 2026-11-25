mod boundary;

use grammar_search::Terms;
use port_core::Op;

use crate::Scope;

#[derive(Clone, Debug, Op)]
pub struct SearchSegments {
    terms: Terms,
    scope: Scope,
}

impl SearchSegments {
    #[must_use]
    pub fn matching(terms: Terms, scope: Scope) -> SearchSegments {
        SearchSegments { terms, scope }
    }
}

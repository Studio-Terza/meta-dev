use std::collections::BTreeMap;

use grammar::search::Terms;
use port_core::Op;
use store_ports::{Query, Statements, StoreFailure};

use crate::Scope;
use crate::boundary::{Hit, Rank, RankRow, SegmentKey};

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

impl Query for SearchSegments {
    type Rows = Vec<Hit>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Hit>, StoreFailure> {
        let first = Hit::matching(self.terms.first(), self.scope, statements)?;
        let rest = self
            .terms
            .rest()
            .iter()
            .map(|term| RankRow::matching(term, statements).map(RankRow::ranked))
            .collect::<Result<Vec<BTreeMap<SegmentKey, Rank>>, StoreFailure>>()?;
        let mut hits: Vec<Hit> = first.into_iter().filter_map(|hit| hit.within(&rest)).collect();
        hits.sort_by(Hit::ranking);
        Ok(hits)
    }
}

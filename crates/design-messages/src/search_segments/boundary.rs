use std::collections::BTreeMap;

use store_ports::{Execute, Query, StoreFailure};

use super::SearchSegments;
use crate::boundary::{Hit, Rank, RankRow};
use design_closure::Keyed;

impl Query for SearchSegments {
    type Rows = Vec<Hit>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<Hit>, StoreFailure> {
        let first = Hit::matching(self.terms.first(), self.scope, statements)?;
        let matched = statements.read_all(self.terms.rest().iter().map(RankRow::matching))?;
        let rest: Vec<BTreeMap<Keyed, Rank>> = matched.into_iter().map(RankRow::ranked).collect();
        let mut hits: Vec<Hit> = first.into_iter().filter_map(|hit| hit.within(&rest)).collect();
        hits.sort_by(Hit::ranking);
        Ok(hits)
    }
}

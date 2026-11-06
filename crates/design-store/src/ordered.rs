use std::collections::BTreeSet;

use design_model::{Segment, SegmentRelation};
use port_core::Op;
use store_ports::{Query, Statements, StoreFailure};

use crate::directions::Columns;

#[derive(Clone, Debug, Op)]
pub struct Ordered {
    from: Segment,
    relation: SegmentRelation,
}

impl Ordered {
    #[must_use]
    pub fn along(relation: SegmentRelation, from: Segment) -> Ordered {
        Ordered { from, relation }
    }
}

impl Query for Ordered {
    type Rows = BTreeSet<Segment>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<BTreeSet<Segment>, StoreFailure> {
        Ok(Columns::ordered(&self.from, self.relation, statements)?.into_iter().collect())
    }
}

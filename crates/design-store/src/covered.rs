use std::collections::BTreeSet;

use design_model::Segment;
use port_core::Op;
use store_ports::{Query, Statements, StoreFailure};
use transcript_ports::Uuid;

use crate::directions::Columns;

#[derive(Clone, Debug, Op)]
pub struct Covered(BTreeSet<Uuid>);

impl FromIterator<Uuid> for Covered {
    fn from_iter<Messages: IntoIterator<Item = Uuid>>(messages: Messages) -> Covered {
        Covered(messages.into_iter().collect())
    }
}

impl Query for Covered {
    type Rows = BTreeSet<Segment>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<BTreeSet<Segment>, StoreFailure> {
        let covered = self
            .0
            .iter()
            .map(|message| Columns::covered(message, statements))
            .collect::<Result<Vec<Vec<Segment>>, StoreFailure>>()?;
        Ok(covered.into_iter().flatten().collect())
    }
}

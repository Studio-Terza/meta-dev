use crate::{Entered, Provenance};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Restager {
    Stage,
}

impl From<Restager> for Provenance {
    fn from(restager: Restager) -> Provenance {
        match restager {
            Restager::Stage => Provenance::Lead(Entered::Stage),
        }
    }
}

use design_model::Provenance;
use newtype::Counter;
use port_core::Op;
use store_ports::{Batch, Statements, Written};

use crate::directions::Annotating;
use crate::{Directed, Refusal};

#[derive(Debug, Op)]
pub struct Imports(Vec<Imported>);

#[derive(Debug)]
pub struct Imported {
    provenance: Provenance,
    directed: Vec<Directed>,
}

impl Imported {
    #[must_use]
    pub fn new(provenance: Provenance, directed: Vec<Directed>) -> Imported {
        Imported { provenance, directed }
    }

    fn written(&self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        let mut annotating = Annotating::new(self.provenance, statements);
        let written = self
            .directed
            .iter()
            .map(|directed| directed.written(&mut annotating))
            .collect::<Result<Vec<Written>, Refusal>>()?;
        Ok(Written::total(written.into_iter()))
    }
}

impl FromIterator<Imported> for Imports {
    fn from_iter<Lines: IntoIterator<Item = Imported>>(lines: Lines) -> Imports {
        Imports(lines.into_iter().collect())
    }
}

impl Batch for Imports {
    type Failure = Refusal;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        let written =
            self.0.iter().map(|imported| imported.written(statements)).collect::<Result<Vec<Written>, Refusal>>()?;
        let unflailed = Annotating::unflailed(statements)?;
        Ok(Written::total(written.into_iter().chain([unflailed])))
    }
}

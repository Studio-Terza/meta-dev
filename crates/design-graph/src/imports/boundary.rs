use design_model::NodeId;
use grammar_notes::Ending;
use newtype::Counter;
use store_ports::{Batch, Execute, Written};

use super::{Imported, Imports};
use crate::directions::Annotating;
use crate::{Directed, Refusal};

impl Batch for Imported {
    type Failure = Refusal;

    fn written(self, statements: &mut impl Execute) -> Result<Written, Refusal> {
        let Imported { provenance, node, parent, directed } = self;
        let mut annotating = Annotating::new(provenance, statements);
        let written = [Directed::Proposed { node, parent }]
            .into_iter()
            .chain(directed)
            .map(|direction| direction.written(&mut annotating))
            .collect::<Result<Vec<Written>, Refusal>>()?;
        Ok(Written::total(written.into_iter()))
    }
}

impl Imported {
    fn ended(node: NodeId, ending: Ending, statements: &mut impl Execute) -> Result<Written, Refusal> {
        statements
            .written(store_ports::query!(
            r#"
INSERT INTO
  notes_endings (node, ending)
VALUES
  (?1, ?2)
"#,
                node: NodeId,
                ending: Ending
            ))
            .map_err(Refusal::Store)
    }
}

impl Batch for Imports {
    type Failure = Refusal;

    fn written(self, statements: &mut impl Execute) -> Result<Written, Refusal> {
        let Imports { ending, lines } = self;
        let last = lines.last().map(|imported| imported.node.identity());
        let written = statements.written_all(lines)?;
        let unflailed = Annotating::unflailed(statements)?;
        let ended = last.map(|node| Imported::ended(node, ending, statements)).transpose()?;
        Ok(Written::total([written, unflailed].into_iter().chain(ended)))
    }
}

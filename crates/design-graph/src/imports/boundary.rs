use design_model::NodeId;
use grammar_notes::Ending;
use store_ports::{Batch, Execute};

use super::{Imported, Imports};
use crate::directions::Annotating;
use crate::{Directed, Refusal};

impl Batch for Imported {
    type Failure = Refusal;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), Refusal> {
        let Imported { provenance, node, parent, directed } = self;
        let mut annotating = Annotating::new(provenance, statements);
        [Directed::Proposed { node, parent }]
            .into_iter()
            .chain(directed)
            .try_for_each(|direction| direction.written(&mut annotating))
    }
}

impl Imported {
    fn ended(node: NodeId, ending: Ending, statements: &mut impl Execute) -> Result<(), Refusal> {
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

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), Refusal> {
        let Imports { ending, lines } = self;
        let last = lines.last().map(|imported| imported.node.identity());
        statements.written_all(lines)?;
        Annotating::unflailed::<()>(statements)?;
        last.map_or(Ok(()), |node| Imported::ended(node, ending, statements))
    }
}

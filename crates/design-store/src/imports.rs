use design_model::{Node, NodeId, Provenance, Vague};
use grammar_notes::Ending;
use newtype::Counter;
use port_core::Op;
use store_ports::{Batch, Statements, Written};

use crate::directions::Annotating;
use crate::{Directed, Refusal};

#[derive(Debug, Op)]
pub struct Imports {
    ending: Ending,
    lines: Vec<Imported>,
}

#[derive(Debug)]
pub struct Imported {
    provenance: Provenance,
    node: Node<Vague>,
    parent: NodeId,
    directed: Vec<Directed>,
}

impl Imported {
    #[must_use]
    pub fn new(provenance: Provenance, node: Node<Vague>, parent: NodeId, directed: Vec<Directed>) -> Imported {
        Imported { provenance, node, parent, directed }
    }

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        let Imported { provenance, node, parent, directed } = self;
        let mut annotating = Annotating::new(provenance, statements);
        let written = [Directed::Proposed { node, parent }]
            .into_iter()
            .chain(directed)
            .map(|directed| directed.written(&mut annotating))
            .collect::<Result<Vec<Written>, Refusal>>()?;
        Ok(Written::total(written.into_iter()))
    }

    fn ended(node: NodeId, ending: Ending, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
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

impl Imports {
    #[must_use]
    pub fn new(ending: Ending, lines: Vec<Imported>) -> Imports {
        Imports { ending, lines }
    }
}

impl Batch for Imports {
    type Failure = Refusal;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        let Imports { ending, lines } = self;
        let last = lines.last().map(|imported| imported.node.identity());
        let written = lines
            .into_iter()
            .map(|imported| imported.written(statements))
            .collect::<Result<Vec<Written>, Refusal>>()?;
        let unflailed = Annotating::unflailed(statements)?;
        let ended = last.map(|node| Imported::ended(node, ending, statements)).transpose()?;
        Ok(Written::total(written.into_iter().chain([unflailed]).chain(ended)))
    }
}

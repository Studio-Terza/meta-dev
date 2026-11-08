use design_model::{Project, Provenance};
use grammar::notes::Ending;
use newtype::Counter;
use port_core::Op;
use store_ports::{Batch, Statements, Written};

use crate::directions::Annotating;
use crate::{Directed, Refusal};

#[derive(Debug, Op)]
pub struct Imports {
    project: Project,
    ending: Ending,
    lines: Vec<Imported>,
}

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

impl Imports {
    #[must_use]
    pub fn new(project: Project, ending: Ending, lines: Vec<Imported>) -> Imports {
        Imports { project, ending, lines }
    }
}

impl Batch for Imports {
    type Failure = Refusal;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        let Imports { project, ending, lines } = self;
        let written =
            lines.iter().map(|imported| imported.written(statements)).collect::<Result<Vec<Written>, Refusal>>()?;
        let unflailed = Annotating::unflailed(statements)?;
        let ended = statements.written(store_ports::query!(
            r#"
INSERT INTO
  notes_endings (project, ending)
VALUES
  (?1, ?2)
"#,
            project: Project,
            ending: Ending
        ))?;
        Ok(Written::total(written.into_iter().chain([unflailed, ended])))
    }
}

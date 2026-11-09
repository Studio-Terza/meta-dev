use crate::{Entered, NotesLine, Provenance, RunId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Restager {
    Apply,
    Role,
    Inheritance,
    Stage,
    Agent(RunId),
    Import(NotesLine),
}

impl From<Restager> for Provenance {
    fn from(restager: Restager) -> Provenance {
        match restager {
            Restager::Apply => Provenance::Lead(Entered::Apply),
            Restager::Role => Provenance::Lead(Entered::Role),
            Restager::Inheritance => Provenance::Lead(Entered::Inheritance),
            Restager::Stage => Provenance::Lead(Entered::Stage),
            Restager::Agent(run) => Provenance::Agent(run),
            Restager::Import(line) => Provenance::Import(line),
        }
    }
}

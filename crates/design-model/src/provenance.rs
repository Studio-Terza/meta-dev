use crate::Entered;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    Lead(Entered),
}

impl From<Entered> for Provenance {
    fn from(entered: Entered) -> Provenance {
        Provenance::Lead(entered)
    }
}

impl From<Provenance> for Entered {
    fn from(provenance: Provenance) -> Entered {
        match provenance {
            Provenance::Lead(entered) => entered,
        }
    }
}

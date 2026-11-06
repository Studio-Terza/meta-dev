use design_model::{Provenance, Standing};

use crate::Directed;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annotation {
    directed: Directed,
    provenance: Provenance,
    standing: Standing,
}

impl Annotation {
    pub(crate) fn new(directed: Directed, provenance: Provenance, standing: Standing) -> Annotation {
        Annotation { directed, provenance, standing }
    }

    #[must_use]
    pub fn directed(&self) -> &Directed {
        &self.directed
    }

    #[must_use]
    pub fn provenance(&self) -> Provenance {
        self.provenance
    }

    #[must_use]
    pub fn standing(&self) -> Standing {
        self.standing
    }
}

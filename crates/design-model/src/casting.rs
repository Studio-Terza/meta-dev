use crate::{Provenance, Role};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Casting {
    role: Role,
    provenance: Provenance,
}

impl Casting {
    #[must_use]
    pub fn new(role: Role, provenance: Provenance) -> Casting {
        Casting { role, provenance }
    }

    #[must_use]
    pub fn role(self) -> Role {
        self.role
    }

    #[must_use]
    pub fn provenance(self) -> Provenance {
        self.provenance
    }
}

use crate::{Entered, Provenance, Role};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Casting {
    role: Role,
    provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inherited {
    role: Role,
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

impl From<Casting> for Inherited {
    fn from(parent: Casting) -> Inherited {
        Inherited { role: parent.role }
    }
}

impl Inherited {
    #[must_use]
    pub fn casting(self, provenance: Provenance) -> Casting {
        match self.role {
            Role::Core => Casting::new(Role::Core, provenance),
            Role::Secondary => Casting::new(Role::Secondary, Provenance::from(Entered::Inheritance)),
        }
    }
}

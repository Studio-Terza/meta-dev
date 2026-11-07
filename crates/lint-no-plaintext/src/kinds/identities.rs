use code_model::{Domain, ItemPath, Model, Resolved};
use lint_layout::{Word, words};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Role {
    Deserializes,
    DeserializesChecked,
}

#[derive(Debug)]
struct Identity {
    resolved: Resolved,
    role: Role,
}

#[derive(Debug)]
pub(super) struct Identities {
    known: Vec<Identity>,
}

impl Identities {
    const SERDE_DESERIALIZE: [Word<'static>; 2] = words!["serde", "Deserialize"];
    const SERDE_DERIVE_DESERIALIZE: [Word<'static>; 2] = words!["serde_derive", "Deserialize"];
    const NEWTYPE_DESERIALIZE: [Word<'static>; 2] = words!["newtype", "Deserialize"];
    const NEWTYPE_CHECKED_DESERIALIZE: [Word<'static>; 2] = words!["newtype", "CheckedDeserialize"];

    pub(super) fn read(model: &Model) -> Identities {
        let known = [
            (Self::SERDE_DESERIALIZE, Role::Deserializes),
            (Self::SERDE_DERIVE_DESERIALIZE, Role::Deserializes),
            (Self::NEWTYPE_DESERIALIZE, Role::Deserializes),
            (Self::NEWTYPE_CHECKED_DESERIALIZE, Role::DeserializesChecked),
        ]
        .into_iter()
        .map(|(path, role)| Identity { resolved: Self::resolved(model, &path), role })
        .collect();
        Identities { known }
    }

    pub(super) fn role(&self, implemented: &Resolved) -> Option<Role> {
        let identified = |known: &Resolved| {
            self.known.iter().find(|identity| identity.resolved == *known).map(|identity| identity.role)
        };
        implemented.known().map_or(Some(Role::Deserializes), identified)
    }

    fn resolved(model: &Model, path: &[Word<'static>]) -> Resolved {
        let path: ItemPath = path.iter().copied().collect();
        let exported = model.exported(Domain::Macros, &path).cloned();
        exported.unwrap_or(Resolved::External(path))
    }
}

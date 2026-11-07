use code_model::{Domain, ItemPath, Model, Resolved};
use lint_layout::{Word, words};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Role {
    Deserializes,
    DeserializesChecked,
    Converts,
}

#[derive(Debug)]
struct Identity {
    resolved: Resolved,
    role: Role,
}

#[derive(Debug)]
pub(super) struct Identities {
    known: Vec<Identity>,
    try_from: ItemPath,
}

impl Identities {
    const SERDE_DESERIALIZE: [Word<'static>; 2] = words!["serde", "Deserialize"];
    const NEWTYPE_DESERIALIZE: [Word<'static>; 2] = words!["newtype", "Deserialize"];
    const NEWTYPE_CHECKED_DESERIALIZE: [Word<'static>; 2] = words!["newtype", "CheckedDeserialize"];
    const TRY_FROM: [Word<'static>; 3] = words!["std", "convert", "TryFrom"];

    pub(super) fn read(model: &Model) -> Identities {
        let known = [
            (Self::SERDE_DESERIALIZE, Role::Deserializes),
            (Self::NEWTYPE_DESERIALIZE, Role::Deserializes),
            (Self::NEWTYPE_CHECKED_DESERIALIZE, Role::DeserializesChecked),
        ]
        .into_iter()
        .map(|(path, role)| Identity { resolved: Self::resolved(model, &path), role })
        .collect();
        Identities { known, try_from: Self::TRY_FROM.into_iter().collect() }
    }

    pub(super) fn role(&self, implemented: &Resolved) -> Option<Role> {
        let known = self.known.iter().find(|identity| identity.resolved == *implemented).map(|identity| identity.role);
        known.or_else(|| implemented.std().filter(|std| **std == self.try_from).map(|_| Role::Converts))
    }

    fn resolved(model: &Model, path: &[Word<'static>]) -> Resolved {
        let path: ItemPath = path.iter().copied().collect();
        let exported = model.exported(Domain::Macros, &path).cloned();
        exported.unwrap_or(Resolved::External(path))
    }
}

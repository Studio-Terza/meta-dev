use std::collections::BTreeMap;

use code_model::{Domain, ItemPath, Model, Resolved};
use lint_layout::{Word, words};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Role {
    Deserializes,
    DeserializesChecked,
    Unresolved,
}

#[derive(Debug)]
pub(super) struct Identities {
    known: BTreeMap<Resolved, Role>,
}

impl Identities {
    const SERDE_PACKAGES: [Word<'static>; 2] = words!["serde", "serde_derive"];
    const SERDE_DERIVE: Word<'static> = Word::new("Deserialize");
    const NEWTYPE_DESERIALIZE: [Word<'static>; 2] = words!["newtype", "Deserialize"];
    const NEWTYPE_CHECKED_DESERIALIZE: [Word<'static>; 2] = words!["newtype", "CheckedDeserialize"];

    pub(super) fn read(model: &Model) -> Identities {
        let serde = Self::SERDE_PACKAGES.into_iter().map(|package| {
            let derive: ItemPath = [package, Self::SERDE_DERIVE].into_iter().collect();
            (Resolved::External(derive), Role::Deserializes)
        });
        let newtype = [
            (Self::NEWTYPE_DESERIALIZE, Role::Deserializes),
            (Self::NEWTYPE_CHECKED_DESERIALIZE, Role::DeserializesChecked),
        ]
        .into_iter()
        .map(|(path, role)| (Self::resolved(model, &path), role));
        Identities { known: serde.chain(newtype).collect() }
    }

    pub(super) fn role(&self, implemented: &Resolved) -> Option<Role> {
        implemented.known().map_or(Some(Role::Unresolved), |known| self.known.get(known).copied())
    }

    fn resolved(model: &Model, path: &[Word<'static>]) -> Resolved {
        let path: ItemPath = path.iter().copied().collect();
        let exported = model.exported(Domain::Macros, &path).cloned();
        exported.unwrap_or(Resolved::External(path))
    }
}

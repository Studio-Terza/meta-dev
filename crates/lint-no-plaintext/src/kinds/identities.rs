use std::collections::BTreeMap;

use code_model::{Domain, External, ItemPath, Model, Resolved};
use lint_layout::{PackageName, Word, words};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Role {
    Deserializes,
    DeserializesChecked,
    Unresolved,
}

#[derive(Debug)]
pub(super) struct Identities {
    exported: BTreeMap<Resolved, Role>,
    packaged: BTreeMap<(PackageName, ItemPath), Role>,
}

impl Identities {
    const SERDE: [Word<'static>; 2] = words!["serde", "serde_derive"];
    const DESERIALIZE: Word<'static> = Word::new("Deserialize");
    const NEWTYPE: Word<'static> = Word::new("newtype");
    const NEWTYPE_DERIVES: [(Word<'static>, Role); 2] =
        [(Self::DESERIALIZE, Role::Deserializes), (Word::new("CheckedDeserialize"), Role::DeserializesChecked)];

    pub(super) fn read(model: &Model) -> Identities {
        let serde = Self::SERDE.into_iter().map(|package| ((package, Self::DESERIALIZE), Role::Deserializes));
        let newtype = Self::NEWTYPE_DERIVES.into_iter().map(|(derive, role)| ((Self::NEWTYPE, derive), role));
        let packaged = serde
            .chain(newtype)
            .map(|((package, derive), role)| ((PackageName::from(package), [derive].into_iter().collect()), role));
        let exported = Self::NEWTYPE_DERIVES.into_iter().filter_map(|(derive, role)| {
            let path: ItemPath = [Self::NEWTYPE, derive].into_iter().collect();
            model.exported(Domain::Macros, &path).cloned().map(|resolved| (resolved, role))
        });
        Identities { exported: exported.collect(), packaged: packaged.collect() }
    }

    pub(super) fn role(&self, implemented: &Resolved) -> Option<Role> {
        implemented.known().map_or(Some(Role::Unresolved), |known| {
            self.exported.get(known).copied().or_else(|| known.external().and_then(|external| self.packaged(external)))
        })
    }

    fn packaged(&self, external: &External) -> Option<Role> {
        let package = external.package()?.clone();
        self.packaged.get(&(package, external.item())).copied()
    }
}

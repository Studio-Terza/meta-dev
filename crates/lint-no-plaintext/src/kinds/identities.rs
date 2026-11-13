use std::collections::{BTreeMap, BTreeSet};

use graph::{Domain, External, ItemPath, Model, Resolved};
use lint_layout::{PackageName, Word, words};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Role {
    Deserializes,
    DeserializesChecked,
    Unresolved,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Packaged {
    package: PackageName,
    item: ItemPath,
}

#[derive(Debug)]
pub(super) struct Identities {
    exported: BTreeMap<Resolved, Role>,
    packaged: BTreeMap<Packaged, Role>,
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
        let packaged = serde.chain(newtype).map(|((package, derive), role)| {
            (Packaged { package: PackageName::from(package), item: [derive].into_iter().collect() }, role)
        });
        let exported = Self::NEWTYPE_DERIVES.into_iter().filter_map(|(derive, role)| {
            let path: ItemPath = [Self::NEWTYPE, derive].into_iter().collect();
            model.exported(Domain::Macros, &path).cloned().map(|resolved| (resolved, role))
        });
        Identities { exported: exported.collect(), packaged: packaged.collect() }
    }

    pub(super) fn role(&self, model: &Model, implemented: &Resolved) -> Option<Role> {
        self.traced(model, implemented, &BTreeSet::new())
    }

    fn traced(&self, model: &Model, resolved: &Resolved, seen: &BTreeSet<Resolved>) -> Option<Role> {
        resolved.known().map_or(Some(Role::Unresolved), |known| {
            self.exported.get(known).copied().or_else(|| self.reexported(model, known.external()?, seen))
        })
    }

    fn reexported(&self, model: &Model, external: &External, seen: &BTreeSet<Resolved>) -> Option<Role> {
        self.packaged(external).or_else(|| {
            let next = model.roots().followed(external)?;
            let deeper: BTreeSet<Resolved> =
                seen.iter().cloned().chain([Resolved::External(external.clone())]).collect();
            deeper.get(&next).map_or_else(|| self.traced(model, &next, &deeper), |_| Some(Role::Unresolved))
        })
    }

    fn packaged(&self, external: &External) -> Option<Role> {
        let packaged =
            Packaged { package: external.package()?.clone(), item: external.below_root().iter().cloned().collect() };
        self.packaged.get(&packaged).copied()
    }
}

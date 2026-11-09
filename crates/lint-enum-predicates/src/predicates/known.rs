use std::collections::BTreeMap;

use lint_layout::Name;
use lint_placement::{CrateRoot, ModulePath};

use super::Kind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Named<'scan> {
    crate_root: &'scan CrateRoot,
    name: &'scan Name,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Placed<'scan> {
    named: Named<'scan>,
    module: &'scan ModulePath,
}

#[derive(Debug, Default)]
pub struct Known<'scan> {
    named: BTreeMap<Named<'scan>, Vec<Kind>>,
    placed: BTreeMap<Placed<'scan>, Kind>,
}

impl<'scan> Known<'scan> {
    pub fn recorded(
        mut self,
        crate_root: &'scan CrateRoot,
        module: &'scan ModulePath,
        name: &'scan Name,
        kind: Kind,
    ) -> Known<'scan> {
        let named = Named { crate_root, name };
        self.named.entry(named).or_default().push(kind);
        self.placed.entry(Placed { named, module }).or_insert(kind);
        self
    }

    pub fn placed(&self, crate_root: &CrateRoot, module: &ModulePath, name: &Name) -> Option<Kind> {
        self.placed.get(&Placed { named: Named { crate_root, name }, module }).copied()
    }

    pub fn enumerated(&self, crate_root: &CrateRoot, name: &Name) -> Option<Kind> {
        let candidates = self.named.get(&Named { crate_root, name }).map_or(&[][..], Vec::as_slice);
        candidates.first().filter(|_| candidates.iter().all(|kind| *kind == Kind::Enum)).map(|_| Kind::Enum)
    }
}

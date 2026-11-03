use std::collections::BTreeSet;

use lint_layout::Name;

#[derive(Debug, Default)]
pub struct CrateReach {
    names: BTreeSet<Name>,
    used: Used,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Used {
    #[default]
    Unused,
    Named,
    Globbed,
}

impl CrateReach {
    pub fn named(&mut self, name: Name) {
        self.names.insert(name);
        self.used = self.used.max(Used::Named);
    }

    pub fn globbed(&mut self) {
        self.used = Used::Globbed;
    }

    pub fn names_only(self) -> CrateReach {
        CrateReach { names: self.names, used: Used::Unused }
    }

    pub fn absorb(&mut self, other: CrateReach) {
        self.names.extend(other.names);
        self.used = self.used.max(other.used);
    }

    pub fn unnamed<'name>(&self, name: &'name Name) -> Option<&'name Name> {
        (self.used == Used::Named).then_some(name).filter(|name| self.names.iter().all(|known| known != *name))
    }
}

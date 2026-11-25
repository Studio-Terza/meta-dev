use std::collections::BTreeSet;

use lint_layout::Name;

#[derive(Debug)]
pub enum Reached {
    Named(Name),
    Glob,
}

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

impl From<Reached> for CrateReach {
    fn from(reached: Reached) -> CrateReach {
        match reached {
            Reached::Named(name) => CrateReach { names: BTreeSet::from([name]), used: Used::Named },
            Reached::Glob => CrateReach { names: BTreeSet::new(), used: Used::Globbed },
        }
    }
}

impl CrateReach {
    pub fn names_only(self) -> CrateReach {
        CrateReach { names: self.names, used: Used::Unused }
    }

    pub fn absorb(&mut self, other: CrateReach) {
        self.names.extend(other.names);
        self.used = self.used.max(other.used);
    }

    pub fn unnamed<'name>(&self, name: &'name Name) -> Option<&'name Name> {
        (self.used == Used::Named)
            .then_some(name)
            .filter(|candidate| self.names.iter().all(|known| known != *candidate))
    }
}

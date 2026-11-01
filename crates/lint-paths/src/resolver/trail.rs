use std::collections::{BTreeMap, BTreeSet};

use crate::module::Module;
use lint_layout::Name;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Globbing {
    #[default]
    Open,
    Spent,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Trail {
    seen: BTreeMap<Module, BTreeSet<Name>>,
    globbing: Globbing,
}

impl Trail {
    pub(crate) fn has_followed(&self, module: &Module, name: &Name) -> bool {
        self.seen.get(module).is_some_and(|names| names.contains(name))
    }

    pub(crate) fn following(&self, module: &Module, name: &Name) -> Trail {
        let mut seen = self.seen.clone();
        seen.entry(module.clone()).or_default().insert(name.clone());
        Trail { seen, globbing: self.globbing }
    }

    pub(crate) fn spent(&self) -> Trail {
        Trail { seen: self.seen.clone(), globbing: Globbing::Spent }
    }

    pub(crate) fn globbing(&self) -> Globbing {
        self.globbing
    }
}

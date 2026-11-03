use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use lint_core::Site;
use type_facts::{Dependency, Held, Via};

use crate::definition::Definition;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Link<'facts, 'file> {
    from: Definition<'facts>,
    to: Definition<'facts>,
    dependency: &'facts Dependency<'file>,
}

impl<'facts, 'file> Link<'facts, 'file> {
    pub(crate) fn new(
        from: Definition<'facts>,
        to: Definition<'facts>,
        dependency: &'facts Dependency<'file>,
    ) -> Link<'facts, 'file> {
        Link { from, to, dependency }
    }

    pub(crate) fn added_to(
        &self,
        mut references: BTreeMap<Definition<'facts>, BTreeSet<Definition<'facts>>>,
    ) -> BTreeMap<Definition<'facts>, BTreeSet<Definition<'facts>>> {
        references.entry(self.from).or_default().insert(self.to);
        references
    }

    pub(crate) fn direct_loop(&self) -> Option<Definition<'facts>> {
        (self.from == self.to && self.dependency.held() == Held::Directly).then_some(self.from)
    }

    pub(crate) fn closing(&self, members: &BTreeSet<Definition<'facts>>) -> Option<Link<'facts, 'file>> {
        (members.contains(&self.from)
            && members.contains(&self.to)
            && (self.from != self.to || self.dependency.held() == Held::Directly))
            .then_some(*self)
    }
}

impl fmt::Display for Link<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} -> {} ({}) at {}",
            self.from,
            self.to,
            AsRef::<Via>::as_ref(self.dependency),
            AsRef::<Site>::as_ref(self.dependency)
        )
    }
}

use std::fmt;

use lint_layout::Name;

use super::cluster::Cluster;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Member<'found> {
    name: &'found Name,
    cluster: Cluster<'found>,
}

impl<'found> Member<'found> {
    pub(crate) fn declared(krate: &'found Name, name: &'found Name) -> Member<'found> {
        Member { name, cluster: Cluster::module(krate, &[]) }
    }

    pub(crate) fn within(self, segments: &'found [Name]) -> Option<Member<'found>> {
        self.cluster.krate().map(|krate| Member { name: self.name, cluster: Cluster::module(krate, segments) })
    }

    pub(crate) fn cluster(&self) -> Cluster<'found> {
        self.cluster
    }

    pub(crate) fn name(&self) -> &'found Name {
        self.name
    }
}

impl fmt::Display for Member<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cluster.krate().into_iter().try_for_each(|krate| write!(formatter, "{krate}"))?;
        self.cluster.segments().iter().chain([self.name]).try_for_each(|segment| write!(formatter, "::{segment}"))
    }
}

use std::borrow::Borrow;
use std::fmt;

use lint_layout::Name;
use lint_placement::CrateName;

use super::cluster::Cluster;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Member<'found> {
    krate: &'found CrateName,
    name: &'found Name,
    segments: &'found [Name],
}

impl<'found> Member<'found> {
    pub(crate) fn new(krate: &'found CrateName, name: &'found Name, segments: &'found [Name]) -> Member<'found> {
        Member { krate, name, segments }
    }

    pub(crate) fn cluster(&self) -> Cluster<'found> {
        Cluster::module(self.krate, self.segments)
    }

    pub(crate) fn name(&self) -> &'found Name {
        self.name
    }

    pub(crate) fn in_crate(&self, krate: &CrateName) -> Option<Member<'found>> {
        Some(*self).filter(|member| member.krate == krate)
    }

    pub(crate) fn crate_named(&self, named: &Name) -> Option<Member<'found>> {
        Some(*self).filter(|member| Borrow::<Name>::borrow(member.krate) == named)
    }
}

impl fmt::Display for Member<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.krate)?;
        self.segments.iter().chain([self.name]).try_for_each(|segment| write!(formatter, "::{segment}"))
    }
}

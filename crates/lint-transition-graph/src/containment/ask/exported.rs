use std::fmt;

use lint_layout::Name;
use newtype::Newtype;

use super::super::Containment;
use super::super::slice::Slice;
use super::listing::{Listed, Names, Placed, Spelled};

#[derive(Debug)]
pub(super) struct Exported<'view, 'found, 'file> {
    view: &'view Containment<'found, 'file>,
    name: &'view Name,
    listed: Listed,
}

impl<'view, 'found, 'file> Exported<'view, 'found, 'file> {
    pub(super) fn new(
        view: &'view Containment<'found, 'file>,
        name: &'view Name,
        listed: Listed,
    ) -> Exported<'view, 'found, 'file> {
        Exported { view, name, listed }
    }
}

impl fmt::Display for Exported<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slices: Vec<&Slice<'_>> =
            self.view.slices().filter(|slice| slice.cluster().named(self.name).is_some()).collect();
        slices.first().map_or_else(|| writeln!(formatter, "  no crate or module named {}", self.name), |_| Ok(()))?;
        slices.into_iter().try_for_each(|slice| {
            writeln!(formatter, "  {}", slice.cluster())?;
            let users: Vec<_> =
                Containment::adjacency(slice.cut().outgoing().iter().map(|link| link.reversed())).into_iter().collect();
            users.first().map_or_else(|| writeln!(formatter, "    none"), |_| Ok(()))?;
            users.iter().take(self.listed.into_inner()).try_for_each(|(export, outside)| {
                let placed = Placed::new(self.view, *export, Spelled::Name);
                writeln!(formatter, "    {placed}; used by {}", outside.iter().copied().collect::<Names<'_>>())
            })?;
            self.listed.more(&users).map_or(Ok(()), |more| writeln!(formatter, "    and {more} more"))
        })
    }
}

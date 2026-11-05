use std::fmt;

use newtype::Newtype;

use super::super::super::Containment;
use super::super::super::member::Member;
use super::super::super::outsider::Outsider;
use super::super::listing::{Listed, Placed, Spelled};

#[derive(Debug)]
pub(super) struct Listing<'view, 'found, 'file> {
    lines: Vec<Line<'view, 'found, 'file>>,
    listed: Listed,
}

#[derive(Debug)]
enum Line<'view, 'found, 'file> {
    Member(Placed<'view, 'found, 'file>),
    Outside(Outsider<'found, 'file>),
}

impl<'view, 'found, 'file> Listing<'view, 'found, 'file> {
    pub(super) fn new(
        view: &'view Containment<'found, 'file>,
        members: impl Iterator<Item = Member<'found>>,
        outsiders: impl Iterator<Item = Outsider<'found, 'file>>,
        listed: Listed,
    ) -> Listing<'view, 'found, 'file> {
        let lines = members
            .map(|member| Line::Member(Placed::new(view, member, Spelled::Name)))
            .chain(outsiders.map(Line::Outside))
            .collect();
        Listing { lines, listed }
    }
}

impl fmt::Display for Line<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Member(placed) => write!(formatter, "{placed}"),
            Line::Outside(outsider) => write!(formatter, "{outsider}"),
        }
    }
}

impl fmt::Display for Listing<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.lines.first().map_or_else(|| writeln!(formatter, "    none"), |_| Ok(()))?;
        self.lines.iter().take(self.listed.into_inner()).try_for_each(|line| writeln!(formatter, "    {line}"))?;
        self.listed.more(&self.lines).map_or(Ok(()), |more| writeln!(formatter, "    and {more} more"))
    }
}

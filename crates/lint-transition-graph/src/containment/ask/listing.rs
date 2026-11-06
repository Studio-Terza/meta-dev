use std::fmt;
use std::num::NonZeroUsize;

use newtype::{Arbitrary, Newtype};

use super::super::Containment;
use super::super::member::Member;
use super::super::outsider::Outsider;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Listed(usize);

impl Listed {
    pub const DEFAULT: Listed = Listed(20);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Spelled {
    Path,
    Name,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Placed<'view, 'found, 'file> {
    view: &'view Containment<'found, 'file>,
    member: Member<'found>,
    spelled: Spelled,
}

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

#[derive(Debug)]
pub(super) struct Names<'found>(Vec<Member<'found>>);

impl<'view, 'found, 'file> Placed<'view, 'found, 'file> {
    pub(super) fn new(
        view: &'view Containment<'found, 'file>,
        member: Member<'found>,
        spelled: Spelled,
    ) -> Placed<'view, 'found, 'file> {
        Placed { view, member, spelled }
    }
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

impl<'found> FromIterator<Member<'found>> for Names<'found> {
    fn from_iter<Members: IntoIterator<Item = Member<'found>>>(members: Members) -> Names<'found> {
        Names(members.into_iter().collect())
    }
}

impl Listed {
    pub(super) fn more<Item>(self, items: &[Item]) -> Option<NonZeroUsize> {
        NonZeroUsize::new(items.len().saturating_sub(self.into_inner()))
    }
}

impl Names<'_> {
    const SHOWN: usize = 8;
}

impl fmt::Display for Line<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Member(placed) => write!(formatter, "{placed}"),
            Line::Outside(outsider) => write!(formatter, "{outsider}"),
        }
    }
}

impl fmt::Display for Placed<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.spelled {
            Spelled::Path => write!(formatter, "{}", self.member),
            Spelled::Name => write!(formatter, "{}", self.member.name()),
        }?;
        self.view.site(self.member).map_or(Ok(()), |site| write!(formatter, " at {site}"))
    }
}

impl fmt::Display for Listing<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.lines.first().map_or_else(|| writeln!(formatter, "    none"), |_| Ok(()))?;
        self.lines.iter().take(self.listed.into_inner()).try_for_each(|line| writeln!(formatter, "    {line}"))?;
        self.listed.more(&self.lines).map_or(Ok(()), |more| writeln!(formatter, "    and {more} more"))
    }
}

impl fmt::Display for Names<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shown: Vec<String> = self.0.iter().take(Self::SHOWN).map(|member| member.name().to_string()).collect();
        formatter.write_str(shown.first().map_or("nothing", |_| ""))?;
        formatter.write_str(&shown.join(", "))?;
        NonZeroUsize::new(self.0.len().saturating_sub(Self::SHOWN))
            .map_or(Ok(()), |more| write!(formatter, " and {more} more"))
    }
}

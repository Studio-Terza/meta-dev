use std::fmt;
use std::num::NonZeroUsize;

use newtype::{Arbitrary, Newtype};

use super::super::Containment;
use super::super::member::Member;

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
    view: &'view Containment<'found, 'file>,
    members: Vec<Member<'found>>,
    listed: Listed,
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
        listed: Listed,
    ) -> Listing<'view, 'found, 'file> {
        Listing { view, members: members.collect(), listed }
    }
}

impl<'found> FromIterator<Member<'found>> for Names<'found> {
    fn from_iter<Members: IntoIterator<Item = Member<'found>>>(members: Members) -> Names<'found> {
        Names(members.into_iter().collect())
    }
}

impl Spelled {
    fn written(self, member: Member<'_>, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Spelled::Path => write!(formatter, "{member}"),
            Spelled::Name => write!(formatter, "{}", member.name()),
        }
    }
}

impl Names<'_> {
    const SHOWN: usize = 8;
}

impl fmt::Display for Placed<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.spelled.written(self.member, formatter)?;
        self.view.site(self.member).map_or(Ok(()), |site| write!(formatter, " at {site}"))
    }
}

impl fmt::Display for Listing<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.members.first().map_or_else(|| writeln!(formatter, "    none"), |_| Ok(()))?;
        self.members
            .iter()
            .take(self.listed.into_inner())
            .try_for_each(|member| writeln!(formatter, "    {}", Placed::new(self.view, *member, Spelled::Name)))?;
        NonZeroUsize::new(self.members.len().saturating_sub(self.listed.into_inner()))
            .map_or(Ok(()), |more| writeln!(formatter, "    and {more} more"))
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

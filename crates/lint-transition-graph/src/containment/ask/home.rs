use std::fmt;

use super::super::member::Member;
use super::super::slice::Slice;
use super::listing::Names;
use super::named::Named;

#[derive(Debug)]
pub(super) struct Home<'view, 'found, 'file> {
    named: Named<'view, 'found, 'file>,
}

impl<'view, 'found, 'file> From<Named<'view, 'found, 'file>> for Home<'view, 'found, 'file> {
    fn from(named: Named<'view, 'found, 'file>) -> Home<'view, 'found, 'file> {
        Home { named }
    }
}

impl<'found> Home<'_, 'found, '_> {
    fn located(&self, formatter: &mut fmt::Formatter<'_>, member: Member<'found>) -> fmt::Result {
        let view = self.named.view();
        let home = member.cluster();
        let users = view.users_within(member);
        let beside: Names<'found> = view
            .slice(home)
            .into_iter()
            .flat_map(Slice::members)
            .filter(|other| other.cluster() == home)
            .filter(|other| **other != member)
            .copied()
            .collect();
        write!(formatter, "    in {home}")?;
        users.map_or(Ok(()), |users| write!(formatter, "; its users lie within {users}"))?;
        writeln!(formatter, "; beside {beside}")
    }
}

impl fmt::Display for Home<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.named.each(formatter, |formatter, member| self.located(formatter, member))
    }
}

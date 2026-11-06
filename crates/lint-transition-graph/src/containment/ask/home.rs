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
    fn beside(&self, member: Member<'found>) -> Names<'found> {
        let home = member.cluster();
        self.named
            .view()
            .slice(home)
            .into_iter()
            .flat_map(Slice::members)
            .filter(|other| other.cluster() == home)
            .filter(|other| **other != member)
            .copied()
            .collect()
    }
}

impl fmt::Display for Home<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.named.found() {
            Err(missing) => write!(formatter, "{missing}"),
            Ok(found) => found.iter().try_for_each(|member| {
                write!(formatter, "  {}\n{}", self.named.placed(*member), Located { home: self, member: *member })
            }),
        }
    }
}

struct Located<'home, 'view, 'found, 'file> {
    home: &'home Home<'view, 'found, 'file>,
    member: Member<'found>,
}

impl fmt::Display for Located<'_, '_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "    in {}", self.member.cluster())?;
        self.home
            .named
            .view()
            .users_within(self.member)
            .map_or(Ok(()), |users| write!(formatter, "; its users lie within {users}"))?;
        writeln!(formatter, "; beside {}", self.home.beside(self.member))
    }
}

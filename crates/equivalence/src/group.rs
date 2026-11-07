use lint_core::Site;

use crate::member::Keyed;

#[derive(Debug)]
pub struct Group<Member: Keyed> {
    key: Member::Key,
    members: Vec<Member>,
}

impl<Member: Keyed> Group<Member> {
    pub(crate) fn new(key: Member::Key, members: Vec<Member>) -> Self {
        Self { key, members }
    }

    #[must_use]
    pub fn key(&self) -> &Member::Key {
        &self.key
    }

    #[must_use]
    pub fn members(&self) -> &[Member] {
        &self.members
    }

    #[must_use]
    pub fn sites<'file>(&self) -> Vec<&Site<'file>>
    where
        Member: AsRef<Site<'file>>,
    {
        let mut sites: Vec<&Site<'file>> = self.members.iter().map(AsRef::as_ref).collect();
        sites.sort();
        sites
    }
}

use std::fmt;

use lint_layout::Name;

use super::super::Containment;
use super::super::member::Member;
use super::listing::{Placed, Spelled};

#[derive(Clone, Copy, Debug)]
pub(super) struct Named<'view, 'found, 'file> {
    view: &'view Containment<'found, 'file>,
    name: &'view Name,
}

impl<'view, 'found, 'file> Named<'view, 'found, 'file> {
    pub(super) fn new(view: &'view Containment<'found, 'file>, name: &'view Name) -> Self {
        Named { view, name }
    }

    pub(super) fn view(&self) -> &'view Containment<'found, 'file> {
        self.view
    }

    pub(super) fn found(&self) -> Result<Vec<Member<'found>>, Missing<'view>> {
        let members: Vec<Member<'found>> = self.view.named(self.name).collect();
        members.first().map(|_| ()).ok_or(Missing(self.name)).map(|()| members)
    }

    pub(super) fn placed(&self, member: Member<'found>) -> Placed<'view, 'found, 'file> {
        Placed::new(self.view, member, Spelled::Path)
    }
}

#[derive(Debug)]
pub(super) struct Missing<'name>(&'name Name);

impl fmt::Display for Missing<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "  no type or trait named {}", self.0)
    }
}

impl fmt::Display for Named<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.found() {
            Ok(members) => members.iter().try_for_each(|member| writeln!(formatter, "  {}", self.placed(*member))),
            Err(missing) => write!(formatter, "{missing}"),
        }
    }
}

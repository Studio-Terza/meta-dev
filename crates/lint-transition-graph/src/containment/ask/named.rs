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
    pub(super) fn new(view: &'view Containment<'found, 'file>, name: &'view Name) -> Named<'view, 'found, 'file> {
        Named { view, name }
    }

    pub(super) fn view(&self) -> &'view Containment<'found, 'file> {
        self.view
    }

    pub(super) fn each(
        &self,
        formatter: &mut fmt::Formatter<'_>,
        mut each: impl FnMut(&mut fmt::Formatter<'_>, Member<'found>) -> fmt::Result,
    ) -> fmt::Result {
        let named: Vec<Member<'found>> = self.view.named(self.name).collect();
        named.first().map_or_else(|| writeln!(formatter, "  no type or trait named {}", self.name), |_| Ok(()))?;
        named.into_iter().try_for_each(|member| {
            writeln!(formatter, "  {}", Placed::new(self.view, member, Spelled::Path))?;
            each(formatter, member)
        })
    }
}

impl fmt::Display for Named<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.each(formatter, |_, _| Ok(()))
    }
}

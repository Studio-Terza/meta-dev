use std::collections::BTreeSet;
use std::iter;

use lint_layout::Name;
use lint_placement::PackageRoot;

#[derive(Debug)]
pub(super) struct Unit<'file> {
    port: &'file PackageRoot,
    members: BTreeSet<&'file PackageRoot>,
    owns: BTreeSet<Name>,
}

impl<'file> Unit<'file> {
    pub(super) fn joined(port: &'file PackageRoot, macros: &BTreeSet<&'file PackageRoot>) -> Option<Unit<'file>> {
        let members: BTreeSet<&PackageRoot> = iter::once(port).chain(macros.iter().copied()).collect();
        let owns = members
            .iter()
            .filter_map(|member| member.owned())
            .flat_map(AsRef::<BTreeSet<Name>>::as_ref)
            .cloned()
            .collect();
        port.owned().map(|_| Unit { port, members, owns })
    }

    pub(super) fn owns(&self) -> &BTreeSet<Name> {
        &self.owns
    }

    pub(super) fn holds(&self, package: &PackageRoot) -> Option<&'file PackageRoot> {
        self.members.get(package).copied()
    }

    pub(super) fn port(&self) -> &'file PackageRoot {
        self.port
    }
}

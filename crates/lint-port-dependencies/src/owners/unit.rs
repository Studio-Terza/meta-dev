use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::iter;

use lint_core::{Site, SourceFile};
use lint_layout::{LineNumber, Name, Owned};
use lint_placement::PackageRoot;
use workspace_metadata::CrateName as PackageName;

#[derive(Debug)]
pub(super) struct Unit<'file> {
    port: &'file PackageRoot,
    declared: &'file Owned,
    file: &'file SourceFile,
    members: BTreeSet<&'file PackageRoot>,
    owns: BTreeSet<Name>,
}

impl<'file> Unit<'file> {
    pub(super) fn joined(
        port: &'file PackageRoot,
        file: &'file SourceFile,
        macros: &BTreeSet<&'file PackageRoot>,
    ) -> Option<Unit<'file>> {
        let declared = port.owned()?;
        let members: BTreeSet<&PackageRoot> = iter::once(port).chain(macros.iter().copied()).collect();
        let owns = members
            .iter()
            .filter_map(|member| member.owned())
            .flat_map(AsRef::<BTreeSet<Name>>::as_ref)
            .cloned()
            .collect();
        Some(Unit { port, declared, file, members, owns })
    }

    pub(super) fn owns(&self) -> &BTreeSet<Name> {
        &self.owns
    }

    pub(super) fn packages(&self) -> BTreeSet<PackageName> {
        self.owns.iter().filter_map(|owned| PackageName::try_from(owned.borrow()).ok()).collect()
    }

    pub(super) fn members(&self) -> &BTreeSet<&'file PackageRoot> {
        &self.members
    }

    pub(super) fn holds(&self, package: &PackageRoot) -> Option<&'file PackageRoot> {
        self.members.get(package).copied()
    }

    pub(super) fn port(&self) -> &'file PackageRoot {
        self.port
    }
}

impl<'file> From<&Unit<'file>> for Site<'file> {
    fn from(unit: &Unit<'file>) -> Site<'file> {
        Site::new(unit.file.manifest_file().as_ref(), *AsRef::<LineNumber>::as_ref(unit.declared))
    }
}

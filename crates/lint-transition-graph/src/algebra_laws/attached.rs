use std::collections::BTreeMap;
use std::path::Path;

use law_facts::{Laws, Package, Property};
use lint_core::{Site, SourceFile};
use lint_layout::Name;

use crate::algebra::{Algebras, Operation};

#[derive(Debug, Default)]
pub(crate) struct Attached<'found> {
    by_operation: BTreeMap<Site<'found>, Vec<Property>>,
}

impl<'found> Attached<'found> {
    pub(crate) fn declared<'file: 'found>(
        files: &'found [SourceFile],
        algebras: &Algebras<'found, 'file>,
    ) -> Attached<'found> {
        let laws = Laws::from(files);
        let packages: BTreeMap<&Path, Package<'found>> =
            files.iter().map(|file| (file.path(), Package::containing(file))).collect();
        let by_operation = algebras
            .operations()
            .filter_map(|operation| {
                let site = *AsRef::<Site<'file>>::as_ref(operation);
                packages.get(site.path()).map(|package| {
                    let names = [AsRef::<Name>::as_ref(operation.acting().subject()), operation.function()];
                    (site, laws.properties_naming(*package, &names).into_iter().collect())
                })
            })
            .collect();
        Attached { by_operation }
    }

    pub(crate) fn of_operation<'file: 'found>(&self, operation: &Operation<'found, 'file>) -> &[Property] {
        self.by_operation.get(AsRef::<Site<'file>>::as_ref(operation)).map_or(&[], Vec::as_slice)
    }
}

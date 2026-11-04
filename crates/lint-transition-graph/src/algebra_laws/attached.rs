use std::collections::BTreeMap;

use law_facts::Property;
use lint_core::Site;
use lint_layout::Name;

use crate::algebra::Operation;
use crate::system::TypeSystem;

#[derive(Debug, Default)]
pub(crate) struct Attached<'found> {
    by_operation: BTreeMap<Site<'found>, Vec<Property>>,
}

impl<'found> Attached<'found> {
    pub(crate) fn declared<'file: 'found>(system: &TypeSystem<'found, 'file>) -> Attached<'found> {
        let declarations = system.declarations();
        let by_operation = system
            .algebras()
            .operations()
            .map(|operation| {
                let site = *AsRef::<Site<'file>>::as_ref(operation);
                let names = [AsRef::<Name>::as_ref(operation.acting().subject()), operation.function()];
                let properties = declarations.properties_naming(site.path(), &names);
                (site, properties.into_iter().filter(|property| *property != Property::Preserves).collect())
            })
            .collect();
        Attached { by_operation }
    }

    pub(crate) fn of_operation<'file: 'found>(&self, operation: &Operation<'found, 'file>) -> &[Property] {
        self.by_operation.get(AsRef::<Site<'file>>::as_ref(operation)).map_or(&[], Vec::as_slice)
    }
}

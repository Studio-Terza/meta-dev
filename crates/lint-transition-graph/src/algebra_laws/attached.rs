use std::collections::BTreeMap;

use law_facts::Property;
use lint_core::Site;
use lint_layout::Name;

use crate::algebra::Operation;
use crate::count::Count;
use crate::system::TypeSystem;

#[derive(Debug, Default)]
pub(crate) struct Attached {
    by_operation: BTreeMap<Count, Vec<Property>>,
}

impl Attached {
    pub(crate) fn declared<'found, 'file: 'found>(system: &TypeSystem<'found, 'file>) -> Attached {
        let declarations = system.declarations();
        let by_operation = system
            .algebras()
            .operations()
            .map(|operation| {
                let site = *AsRef::<Site<'file>>::as_ref(operation);
                let names = [AsRef::<Name>::as_ref(operation.acting().subject()), operation.function()];
                let properties = declarations.properties_naming(site.path(), &names);
                (
                    operation.ordinal(),
                    properties.into_iter().filter(|property| *property != Property::Preserves).collect(),
                )
            })
            .collect();
        Attached { by_operation }
    }

    pub(crate) fn of_operation(&self, operation: &Operation<'_, '_>) -> &[Property] {
        self.by_operation.get(&operation.ordinal()).map_or(&[], Vec::as_slice)
    }
}

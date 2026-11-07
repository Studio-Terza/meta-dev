use std::collections::BTreeSet;

use syn::visit::{self, Visit};
use syn::{Item, TypeParam};

use lint_layout::{Name, UsePath};

#[derive(Debug, Default)]
pub struct Parameters(BTreeSet<Name>);

impl Parameters {
    pub fn of_item(item: &Item) -> Self {
        let mut parameters = Self::default();
        parameters.visit_item(item);
        parameters
    }

    pub fn unbound(&self, paths: Vec<UsePath>) -> Vec<UsePath> {
        paths.into_iter().filter(|path| path.segments().first().and_then(|head| self.0.get(head)).is_none()).collect()
    }
}

impl Visit<'_> for Parameters {
    fn visit_type_param(&mut self, parameter: &TypeParam) {
        self.0.insert(Name::from(&parameter.ident));
        visit::visit_type_param(self, parameter);
    }
}

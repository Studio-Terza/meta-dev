use std::collections::BTreeMap;

use syn::visit::{self, Visit};
use syn::{Block, Expr, File, ItemConst, Local, Pat};

use lint_layout::Name;

#[derive(Debug, Default)]
pub struct Bindings(BTreeMap<Name, Expr>);

impl Bindings {
    pub fn constants(file: &File) -> Bindings {
        let mut constants = Constants(Bindings::default());
        constants.visit_file(file);
        constants.0
    }

    pub fn locals(block: &Block) -> Bindings {
        let mut locals = Locals(Bindings::default());
        locals.visit_block(block);
        locals.0
    }

    pub fn bound(&self, name: &Name) -> Option<&Expr> {
        self.0.get(name)
    }
}

struct Constants(Bindings);

impl<'ast> Visit<'ast> for Constants {
    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.0.0.insert(Name::from(&item.ident), (*item.expr).clone());
        visit::visit_item_const(self, item);
    }
}

struct Locals(Bindings);

impl<'ast> Visit<'ast> for Locals {
    fn visit_local(&mut self, local: &'ast Local) {
        let bound = match (&local.pat, &local.init) {
            (Pat::Ident(name), Some(init)) => Some((Name::from(&name.ident), (*init.expr).clone())),
            _ => None,
        };
        self.0.0.extend(bound);
        visit::visit_local(self, local);
    }
}

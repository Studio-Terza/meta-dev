use std::collections::BTreeMap;

use syn::visit::{self, Visit};
use syn::{Block, Expr, File, ItemConst, Local, Pat};

use lint_layout::Name;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Scope {
    #[default]
    Items,
    Body,
}

#[derive(Debug, Default)]
pub struct Bindings {
    bound: BTreeMap<Name, Expr>,
    scope: Scope,
}

impl Bindings {
    pub fn constants(file: &File) -> Bindings {
        let mut constants = Bindings::default();
        constants.visit_file(file);
        constants
    }

    pub fn locals(block: &Block) -> Bindings {
        let mut locals = Bindings { bound: BTreeMap::new(), scope: Scope::Body };
        locals.visit_block(block);
        locals
    }

    pub fn bound(&self, name: &Name) -> Option<&Expr> {
        self.bound.get(name)
    }
}

impl<'ast> Visit<'ast> for Bindings {
    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.bound.extend((self.scope == Scope::Items).then(|| (Name::from(&item.ident), (*item.expr).clone())));
        visit::visit_item_const(self, item);
    }

    fn visit_local(&mut self, local: &'ast Local) {
        let bound = match (&local.pat, &local.init, self.scope) {
            (Pat::Ident(name), Some(init), Scope::Body) => Some((Name::from(&name.ident), (*init.expr).clone())),
            _ => None,
        };
        self.bound.extend(bound);
        visit::visit_local(self, local);
    }
}

use std::collections::BTreeMap;

use syn::visit::{self, Visit};
use syn::{Block, Expr, File, ImplItemFn, ItemConst, ItemFn, Signature};

use lint_core::Syntax;
use lint_layout::Name;

use super::trace::Locals;

#[derive(Debug)]
pub(crate) struct Helper<'file> {
    inputs: Vec<Name>,
    block: &'file Block,
}

impl<'file> Helper<'file> {
    fn declared(signature: &'file Signature, block: &'file Block) -> Helper<'file> {
        let inputs = signature
            .inputs
            .iter()
            .filter_map(Syntax::typed)
            .filter_map(|typed| Syntax::pat_ident(&typed.pat))
            .map(|input| Name::from(&input.ident))
            .collect();
        Helper { inputs, block }
    }

    pub(crate) fn called(&self, arguments: &[&'file Expr], caller: &Locals<'file>) -> Locals<'file> {
        self.inputs.iter().zip(arguments).fold(Locals::default(), |locals, (input, argument)| {
            locals.binding(input.clone(), caller.resolved(argument))
        })
    }

    pub(crate) fn visited(&self, visitor: &mut impl Visit<'file>) {
        visitor.visit_block(self.block);
    }
}

#[derive(Debug, Default)]
pub(crate) struct Scope<'file> {
    constants: BTreeMap<Name, &'file Expr>,
    helpers: BTreeMap<Name, Vec<Helper<'file>>>,
}

impl<'file> Scope<'file> {
    pub(crate) fn of_file(file: &'file File) -> Scope<'file> {
        let mut scope = Scope::default();
        scope.visit_file(file);
        scope
    }

    pub(crate) fn constant(&self, name: &Name) -> Option<&'file Expr> {
        self.constants.get(name).copied()
    }

    pub(crate) fn helpers(&self, name: &Name) -> &[Helper<'file>] {
        self.helpers.get(name).map_or(&[], Vec::as_slice)
    }

    fn register(&mut self, signature: &'file Signature, block: &'file Block) {
        self.helpers.entry(Name::from(&signature.ident)).or_default().push(Helper::declared(signature, block));
    }
}

impl<'file> Visit<'file> for Scope<'file> {
    fn visit_item_const(&mut self, item: &'file ItemConst) {
        self.constants.insert(Name::from(&item.ident), &item.expr);
    }

    fn visit_item_fn(&mut self, item: &'file ItemFn) {
        self.register(&item.sig, &item.block);
        visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'file ImplItemFn) {
        self.register(&item.sig, &item.block);
        visit::visit_impl_item_fn(self, item);
    }
}

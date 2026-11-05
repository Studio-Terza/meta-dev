use std::collections::BTreeMap;
use std::mem;

use syn::visit::{self, Visit};
use syn::{Block, Expr, File, ImplItemFn, ItemConst, ItemFn, ItemImpl, Path, Signature};

use lint_core::Syntax;
use lint_layout::Name;

use super::trace::Locals;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Target {
    owner: Option<Name>,
    name: Name,
}

impl Target {
    const SELF: &'static str = "Self";

    pub(crate) fn new(owner: Option<Name>, name: Name) -> Target {
        Target { owner, name }
    }

    pub(crate) fn called(path: &Path, owner: Option<&Name>) -> Option<Target> {
        let names: Vec<Name> = path.segments.iter().map(|segment| Name::from(&segment.ident)).collect();
        let resolved = |qualifier: &Name| owner.filter(|_| *qualifier == Self::SELF).unwrap_or(qualifier).clone();
        names
            .split_last()
            .filter(|(_, qualifiers)| qualifiers.len() <= 1)
            .map(|(name, qualifiers)| Target::new(qualifiers.first().map(resolved), name.clone()))
    }
}

#[derive(Debug)]
pub(crate) struct Helper<'file> {
    target: Target,
    inputs: Vec<Option<Name>>,
    block: &'file Block,
}

impl<'file> Helper<'file> {
    fn declared(owner: Option<Name>, signature: &'file Signature, block: &'file Block) -> Helper<'file> {
        let inputs = signature
            .inputs
            .iter()
            .filter_map(Syntax::typed)
            .map(|typed| Syntax::pat_ident(&typed.pat).map(|input| Name::from(&input.ident)))
            .collect();
        Helper { target: Target::new(owner, Name::from(&signature.ident)), inputs, block }
    }

    pub(crate) fn matching(&self, arguments: &[&Expr]) -> Option<&Helper<'file>> {
        Some(self).filter(|helper| helper.inputs.len() == arguments.len())
    }

    pub(crate) fn target(&self) -> &Target {
        &self.target
    }

    pub(crate) fn owner(&self) -> Option<Name> {
        self.target.owner.clone()
    }

    pub(crate) fn called(&self, arguments: &[&'file Expr], caller: &Locals<'file>) -> Locals<'file> {
        self.inputs.iter().zip(arguments).fold(Locals::default(), |locals, (input, argument)| {
            input.iter().fold(locals, |locals, name| locals.binding(name.clone(), caller.resolved(argument)))
        })
    }

    pub(crate) fn visited(&self, visitor: &mut impl Visit<'file>) {
        visitor.visit_block(self.block);
    }
}

#[derive(Debug, Default)]
pub(crate) struct Scope<'file> {
    constants: BTreeMap<Name, &'file Expr>,
    helpers: BTreeMap<Target, Vec<Helper<'file>>>,
    owner: Option<Name>,
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

    pub(crate) fn helpers(&self, path: &Path, owner: Option<&Name>, arguments: &[&Expr]) -> Vec<&Helper<'file>> {
        Target::called(path, owner)
            .and_then(|key| self.helpers.get(&key))
            .into_iter()
            .flatten()
            .filter_map(|helper| helper.matching(arguments))
            .collect()
    }

    fn register(&mut self, signature: &'file Signature, block: &'file Block) {
        let owner = self.owner.clone();
        let helper = Helper::declared(owner, signature, block);
        let target = helper.target().clone();
        self.helpers.entry(target).or_default().push(helper);
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

    fn visit_item_impl(&mut self, item: &'file ItemImpl) {
        let outer = mem::replace(&mut self.owner, Name::of_type(&item.self_ty));
        visit::visit_item_impl(self, item);
        self.owner = outer;
    }

    fn visit_impl_item_fn(&mut self, item: &'file ImplItemFn) {
        Some(&item.sig).filter(|signature| signature.receiver().is_none()).iter().for_each(|signature| {
            self.register(signature, &item.block);
        });
        visit::visit_impl_item_fn(self, item);
    }
}

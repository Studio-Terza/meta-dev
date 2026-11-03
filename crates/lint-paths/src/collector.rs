use syn::visit::{self, Visit};
use syn::{Ident, ItemImpl, ItemMod, ItemType, ItemUse, Type, UseTree};

use crate::module::Module;
use crate::scopes::Scopes;
use lint_core::SourceFile;
use lint_layout::{Name, UsePath};

pub(crate) struct Collector<'file> {
    scopes: Scopes<'file>,
    module: Module<'file>,
}

impl<'file> Collector<'file> {
    const SELF: &'static str = "self";
    const DROP: &'static str = "Drop";

    pub(crate) fn collect(files: &[&'file SourceFile]) -> Scopes<'file> {
        files.iter().fold(Scopes::default(), |scopes, file| {
            let module = Module::of_file(file);
            let mut collector = Collector { scopes, module };
            collector.scopes.declare(&collector.module);
            collector.visit_file(file.syntax());
            collector.scopes
        })
    }

    fn imported(&mut self, prefix: UsePath, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => self.imported(prefix.joined(&[Name::from(&path.ident)]), &path.tree),
            UseTree::Name(name) => self.bound(prefix, &name.ident, None),
            UseTree::Rename(name) => self.bound(prefix, &name.ident, Some(&name.rename)),
            UseTree::Glob(_) => self.scopes.glob(&self.module, prefix),
            UseTree::Group(group) => group.items.iter().for_each(|item| self.imported(prefix.clone(), item)),
        }
    }

    fn bound(&mut self, prefix: UsePath, ident: &Ident, rename: Option<&Ident>) {
        let target = if ident == Self::SELF { prefix } else { prefix.joined(&[Name::from(ident)]) };
        let local = rename.map(Name::from).or_else(|| target.segments().last().cloned());
        self.scopes.import(&self.module, local, target);
    }

    fn dropped(item: &ItemImpl) -> Option<Name> {
        item.trait_.as_ref().and_then(|(_, path, _)| path.segments.last()).filter(|last| last.ident == Self::DROP)?;
        match item.self_ty.as_ref() {
            Type::Path(path) => path.path.segments.last().map(|segment| Name::from(&segment.ident)),
            _ => None,
        }
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.module.enter(Name::from(&module.ident));
        self.scopes.declare(&self.module);
        visit::visit_item_mod(self, module);
        self.module.leave();
    }

    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.imported(UsePath::default(), &item.tree);
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        let target = match item.ty.as_ref() {
            Type::Path(path) => UsePath::from(&path.path),
            _ => UsePath::default(),
        };
        self.scopes.import(&self.module, Some(Name::from(&item.ident)), target);
        visit::visit_item_type(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.scopes.guard(&self.module, Self::dropped(item));
        visit::visit_item_impl(self, item);
    }
}

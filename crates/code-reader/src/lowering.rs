use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, File, Item, ItemEnum, ItemImpl, ItemMacro, ItemMod, ItemStruct, ItemUnion, Path, Stmt, Token,
};

use code_model::{Found, Implementation, Origin};
use lint_core::SourceFile;
use lint_layout::{LineNumber, Name};

mod boundary;
mod macro_impls;
mod scopes;

use boundary::SelfType;
use macro_impls::MacroImpls;
use scopes::{Reach, Scopes};

#[derive(Debug)]
pub(crate) struct Lowering {
    found: Vec<Found<Implementation>>,
    scopes: Scopes,
}

impl Lowering {
    const DERIVE: &'static str = "derive";

    pub(crate) fn lowered(file: &SourceFile) -> Vec<Found<Implementation>> {
        let syntax: &File = file.syntax();
        let mut lowering = Lowering { found: Vec::new(), scopes: Scopes::new(&syntax.items) };
        lowering.visit_file(syntax);
        lowering.found
    }

    fn declared(statement: &Stmt) -> Option<&Item> {
        match statement {
            Stmt::Item(item) => Some(item),
            _ => None,
        }
    }

    fn derived(&mut self, attributes: &[Attribute], subject: &syn::Ident) {
        let derived: Vec<Found<Implementation>> = attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::DERIVE))
            .filter_map(|attribute| attribute.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated).ok())
            .flatten()
            .map(|path| {
                Found::new(
                    LineNumber::from(path.segments.first().map_or(subject.span(), |first| first.ident.span())),
                    Implementation::new(Name::from(subject), Some(self.scopes.resolved(&path)), Origin::Derived),
                )
            })
            .collect();
        self.found.extend(derived);
    }
}

impl<'ast> Visit<'ast> for Lowering {
    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        self.scopes.enter(Reach::Module, item.content.iter().flat_map(|(_, items)| items));
        visit::visit_item_mod(self, item);
        self.scopes.leave();
    }

    fn visit_block(&mut self, block: &'ast Block) {
        self.scopes.enter(Reach::Block, block.stmts.iter().filter_map(Self::declared));
        visit::visit_block(self, block);
        self.scopes.leave();
    }

    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        let subject = SelfType::written(&block.self_ty);
        let implemented = block.trait_.as_ref().map(|(_, path, _)| self.scopes.resolved(path));
        let line = LineNumber::from(block.impl_token.span);
        self.found.push(Found::new(line, Implementation::new(subject, implemented, Origin::Written)));
        visit::visit_item_impl(self, block);
    }

    fn visit_item_macro(&mut self, item: &'ast ItemMacro) {
        self.found.extend(MacroImpls::in_macro(&item.mac, &self.scopes));
        visit::visit_item_macro(self, item);
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.derived(&item.attrs, &item.ident);
        visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.derived(&item.attrs, &item.ident);
        visit::visit_item_enum(self, item);
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        self.derived(&item.attrs, &item.ident);
        visit::visit_item_union(self, item);
    }
}

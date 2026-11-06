use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Attribute, File, ItemEnum, ItemImpl, ItemMacro, ItemStruct, ItemUnion, Path, Token};

use code_model::{Found, Implementation, Origin, TraitPath};
use lint_core::SourceFile;
use lint_layout::{LineNumber, Name};

mod macro_impls;
mod renames;

use macro_impls::MacroImpls;
use renames::Renames;

#[derive(Debug, Default)]
pub(crate) struct Lowering<'ast> {
    found: Vec<Found<Implementation>>,
    renames: Renames<'ast>,
}

impl<'ast> Lowering<'ast> {
    const DERIVE: &'static str = "derive";

    pub(crate) fn lowered(file: &'ast SourceFile) -> Vec<Found<Implementation>> {
        let syntax: &File = file.syntax();
        let mut lowering = Lowering { found: Vec::new(), renames: Renames::in_file(syntax) };
        lowering.visit_file(syntax);
        lowering.found
    }

    fn trait_path(&self, path: &Path) -> TraitPath {
        let length = path.segments.len();
        path.segments
            .iter()
            .enumerate()
            .map(|(at, segment)| {
                (at + 1 == length)
                    .then_some(&segment.ident)
                    .map_or_else(|| Name::from(&segment.ident), |ident| self.renames.resolved(ident))
            })
            .collect()
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
                    Implementation::new(Name::from(subject), Some(self.trait_path(&path)), Origin::Derived),
                )
            })
            .collect();
        self.found.extend(derived);
    }
}

impl<'ast> Visit<'ast> for Lowering<'ast> {
    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        let subject = Name::of_type(&block.self_ty).unwrap_or_default();
        let implemented = block.trait_.as_ref().map(|(_, path, _)| self.trait_path(path));
        let line = LineNumber::from(block.impl_token.span);
        self.found.push(Found::new(line, Implementation::new(subject, implemented, Origin::Written)));
        visit::visit_item_impl(self, block);
    }

    fn visit_item_macro(&mut self, item: &'ast ItemMacro) {
        self.found.extend(MacroImpls::in_macro(&item.mac));
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

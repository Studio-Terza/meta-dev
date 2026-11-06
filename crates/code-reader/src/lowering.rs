use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Attribute, ItemEnum, ItemImpl, ItemStruct, ItemUnion, Path, Token};

use code_model::{Found, Implementation, Origin, TraitPath};
use lint_core::SourceFile;
use lint_layout::{LineNumber, Name};

#[derive(Debug, Default)]
pub(crate) struct Lowering {
    found: Vec<Found<Implementation>>,
}

impl Lowering {
    const DERIVE: &'static str = "derive";

    pub(crate) fn lowered(file: &SourceFile) -> Vec<Found<Implementation>> {
        let mut lowering = Lowering::default();
        lowering.visit_file(file.syntax());
        lowering.found
    }

    fn trait_path(path: &Path) -> TraitPath {
        path.segments.iter().map(|segment| Name::from(&segment.ident)).collect()
    }

    fn derived(&mut self, attributes: &[Attribute], subject: &syn::Ident) {
        let derived = attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::DERIVE))
            .filter_map(|attribute| attribute.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated).ok())
            .flatten()
            .map(|path| {
                Found::new(
                    LineNumber::from(path.segments.first().map_or(subject.span(), |first| first.ident.span())),
                    Implementation::new(Name::from(subject), Some(Self::trait_path(&path)), Origin::Derived),
                )
            });
        self.found.extend(derived);
    }
}

impl<'ast> Visit<'ast> for Lowering {
    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        let subject = Name::of_type(&block.self_ty).unwrap_or_default();
        let implemented = block.trait_.as_ref().map(|(_, path, _)| Self::trait_path(path));
        let line = LineNumber::from(block.impl_token.span);
        self.found.push(Found::new(line, Implementation::new(subject, implemented, Origin::Written)));
        visit::visit_item_impl(self, block);
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

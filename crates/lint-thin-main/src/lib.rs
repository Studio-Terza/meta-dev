mod finding;

use finding::Finding;
use lint_core::Lint;
use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Item, ItemFn};

use lint_core::{Headline, Report, Site, SourceFile, Violation};
use lint_layout::LineNumber;
use lint_placement::Role;

#[derive(Debug)]
pub struct ThinMain;

impl Lint for ThinMain {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new("a binary is a thin main over its library"),
            files
                .iter()
                .filter(|file| file.placement().role() == Role::BinaryRoot)
                .flat_map(|file| {
                    file.syntax().items.iter().filter(|item| !Self::is_thin(item)).map(|item| {
                        Violation::new(Site::new(file.path(), LineNumber::from(Self::keyword(item))), Finding::Logic)
                    })
                })
                .collect(),
        )
    }
}

impl ThinMain {
    fn keyword(item: &Item) -> Span {
        match item {
            Item::Use(found) => found.use_token.span,
            Item::Fn(found) => found.sig.fn_token.span,
            Item::Struct(found) => found.struct_token.span,
            Item::Enum(found) => found.enum_token.span,
            Item::Mod(found) => found.mod_token.span,
            Item::Impl(found) => found.impl_token.span,
            Item::Trait(found) => found.trait_token.span,
            Item::Const(found) => found.const_token.span,
            Item::Static(found) => found.static_token.span,
            Item::Type(found) => found.type_token.span,
            _ => item.span(),
        }
    }

    fn is_thin(item: &Item) -> bool {
        match item {
            Item::Use(_) => true,
            Item::Fn(ItemFn { sig, .. }) => sig.ident == "main",
            _ => false,
        }
    }
}

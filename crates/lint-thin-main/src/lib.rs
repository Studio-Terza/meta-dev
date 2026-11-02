mod body_items;
mod finding;

use body_items::BodyItems;

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
                    let items = &file.syntax().items;
                    let inside_main = Self::mains(items).flat_map(BodyItems::declared_in);
                    items.iter().filter(|item| Self::thin(item).is_none()).chain(inside_main).map(|item| {
                        Violation::new(Site::new(file.path(), LineNumber::from(Self::keyword(item))), Finding::Logic)
                    })
                })
                .collect(),
        )
    }
}

impl ThinMain {
    fn mains(items: &[Item]) -> impl Iterator<Item = &ItemFn> {
        items.iter().filter_map(|item| match item {
            Item::Fn(main) if Self::thin(item).is_some() => Some(main),
            _ => None,
        })
    }

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

    fn thin(item: &Item) -> Option<&Item> {
        match item {
            Item::Use(_) => Some(item),
            Item::Fn(ItemFn { sig, .. }) if sig.ident == "main" => Some(item),
            _ => None,
        }
    }
}

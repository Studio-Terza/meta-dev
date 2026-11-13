mod body_items;
mod finding;

use body_items::BodyItems;

use finding::Finding;
use lint_file::{FileLint, Reads};
use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Item, ItemFn};

use lint_core::{Headline, Site, SourceFile, Violation};
use lint_layout::LineNumber;
use lint_placement::Role;

#[derive(Debug)]
pub struct ThinMain;

impl FileLint for ThinMain {
    const HEADLINE: Headline = Headline::new("a binary is a thin main over its library");
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let binary = Some(file).filter(|found| found.head().placement().role() == Role::BinaryRoot);
        binary.map_or_default(|root| {
            let items = &root.syntax().items;
            let inside_main = Self::mains(items).flat_map(|main| BodyItems::declared_in(root, main));
            let outside = items.iter().filter(|item| Self::thin(item).is_none()).map(Self::keyword);
            let logic = outside.chain(inside_main);
            logic
                .map(|span| Violation::new(Site::new(root.head().path(), LineNumber::from(span)), Finding::Logic))
                .collect()
        })
    }
}

impl ThinMain {
    fn mains(items: &[Item]) -> impl Iterator<Item = &ItemFn> {
        items.iter().filter(|item| Self::thin(item).is_some()).filter_map(Self::function)
    }

    fn function(item: &Item) -> Option<&ItemFn> {
        match item {
            Item::Fn(main) => Some(main),
            _ => None,
        }
    }

    fn keyword(item: &Item) -> Span {
        match item {
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
            Item::Fn(ItemFn { sig, .. }) => (sig.ident == "main").then_some(item),
            _ => None,
        }
    }
}

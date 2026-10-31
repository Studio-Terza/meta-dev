mod finding;

use finding::Finding;
use syn::{Ident, Item, ItemFn, ItemMod};

use lint_core::LineNumber;
use lint_core::Name;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;

#[derive(Debug)]
pub struct NoFreeFns;

impl NoFreeFns {
    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "free functions are not allowed; make them associated functions or methods",
            files
                .iter()
                .flat_map(|file| {
                    Self::free_fns(&file.syntax().items).into_iter().map(move |name| {
                        Violation::new(
                            Site::new(file.path(), LineNumber::from(name.span())),
                            Finding::FreeFn { name: Name::from(&name) },
                        )
                    })
                })
                .collect(),
        )
    }

    fn free_fns(items: &[Item]) -> Vec<Ident> {
        items
            .iter()
            .flat_map(|item| match item {
                Item::Fn(function) if !Self::is_exempt(function) => {
                    vec![function.sig.ident.clone()]
                }
                Item::Mod(module) => Self::nested(module),
                _ => Vec::new(),
            })
            .collect()
    }

    fn nested(module: &ItemMod) -> Vec<Ident> {
        module.content.as_ref().map_or_else(Vec::new, |(_, items)| Self::free_fns(items))
    }

    fn is_exempt(function: &ItemFn) -> bool {
        function.sig.ident == "main" || function.attrs.iter().any(|attribute| attribute.path().is_ident("test"))
    }
}

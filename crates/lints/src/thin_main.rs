mod finding;

use finding::Finding;
use syn::spanned::Spanned;
use syn::{Item, ItemFn};

use lint_core::{LineNumber, Report, Site, SourceFile, Violation};

#[derive(Debug)]
pub struct ThinMain;

impl ThinMain {
    const BINARY_ROOT: &str = "src/main.rs";

    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "a binary is a thin main over its library",
            files
                .iter()
                .filter(|file| file.path().ends_with(Self::BINARY_ROOT))
                .flat_map(|file| {
                    file.syntax().items.iter().filter(|item| !Self::is_thin(item)).map(|item| {
                        Violation::new(Site::new(file.path(), LineNumber::from(item.span())), Finding::Logic)
                    })
                })
                .collect(),
        )
    }

    fn is_thin(item: &Item) -> bool {
        match item {
            Item::Use(_) => true,
            Item::Fn(ItemFn { sig, .. }) => sig.ident == "main",
            _ => false,
        }
    }
}

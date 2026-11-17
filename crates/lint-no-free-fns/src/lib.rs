mod finding;

use finding::Finding;
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Facts, Finds, Fold, Folding, Kind, Visit};
use std::path::Path as StdPath;

use syntax::{Ident, ItemFn, Path};

use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::LineNumber;

#[derive(Debug)]
pub struct NoFreeFns;

impl NoFreeFns {
    const REQUIRED_FREE: [&'static str; 4] = ["test", "proc_macro", "proc_macro_derive", "proc_macro_attribute"];

    fn exemption(function: &ItemFn) -> Option<&Ident> {
        Some(&function.sig.ident)
            .filter(|name| **name == "main")
            .or_else(|| function.attrs.iter().find_map(|attribute| Self::required_free(attribute.path())))
    }

    fn required_free(attribute: &Path) -> Option<&Ident> {
        attribute
            .segments
            .last()
            .map(|segment| &segment.ident)
            .filter(|name| Self::REQUIRED_FREE.iter().any(|required| **name == required))
    }
}

impl SweptLint for NoFreeFns {
    const HEADLINE: Headline =
        Headline::new("free functions are not allowed; make them associated functions or methods");
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(Folding::from(FreeFns { path: AsRef::<StdPath>::as_ref(file.head()) }))
    }
}

#[derive(Debug)]
struct FreeFns<'file> {
    path: &'file StdPath,
}

impl<'file> Fold for FreeFns<'file> {
    type Found = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemFn, Kind::Macro]
    }

    fn on_item_fn(&self, found: Self::Found, function: &ItemFn, _: &Facts<'_, ItemFn>) -> Visit<Self::Found> {
        let name = &function.sig.ident;
        let free = NoFreeFns::exemption(function).map_or_else(
            || Some(Violation::new(Site::new(self.path, LineNumber::from(name.span())), Finding::FreeFn { name })),
            |_| None,
        );
        Visit::descending(found.into_iter().chain(free).collect())
    }

    fn on_macro(&self, found: Self::Found, _: &syntax::Macro, _: &Facts<'_, syntax::Macro>) -> Visit<Self::Found> {
        Visit::new(found, Descend::Expand)
    }
}

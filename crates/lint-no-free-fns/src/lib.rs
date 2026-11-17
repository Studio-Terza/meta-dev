mod finding;

use finding::Finding;
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Concludes, Contribute, Contributed, Contributing, Descend, Facts, Finds, Kind};
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
        Box::new(Contributing::new(FreeFns { path: AsRef::<StdPath>::as_ref(file.head()) }))
    }
}

#[derive(Debug)]
struct FreeFns<'file> {
    path: &'file StdPath,
}

impl<'file> Contribute for FreeFns<'file> {
    type Open = ();
    type Part = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemFn, Kind::Macro]
    }

    fn on_item_fn(&self, function: &ItemFn, _: &Facts<'_, ItemFn>) -> Contributed<(), Vec<Violation<'file>>> {
        let name = &function.sig.ident;
        let free = NoFreeFns::exemption(function).map_or_else(
            || Some(Violation::new(Site::new(self.path, LineNumber::from(name.span())), Finding::FreeFn { name })),
            |_| None,
        );
        Contributed::parted(free.into_iter().collect())
    }

    fn on_macro(&self, _: &syntax::Macro, _: &Facts<'_, syntax::Macro>) -> Contributed<(), Vec<Violation<'file>>> {
        Contributed::descending(Descend::Expand)
    }
}

impl<'file> Concludes<'file> for FreeFns<'file> {
    fn violations(self, (): (), found: Vec<Violation<'file>>) -> Vec<Violation<'file>> {
        found
    }
}

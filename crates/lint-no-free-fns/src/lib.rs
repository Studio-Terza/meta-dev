mod finding;

use finding::Finding;
use lint_core::{Headline, Lint};
use std::path::Path as StdPath;
use syn::visit::Visit;

use syn::{Ident, ItemFn, Path};

use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::LineNumber;

#[derive(Debug)]
pub struct NoFreeFns;

impl Lint for NoFreeFns {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new("free functions are not allowed; make them associated functions or methods"),
            files.iter().flat_map(|file| Self::free_fns(file.path(), file.syntax())).collect(),
        )
    }
}

impl NoFreeFns {
    const REQUIRED_FREE: [&'static str; 4] = ["test", "proc_macro", "proc_macro_derive", "proc_macro_attribute"];

    fn free_fns(path: &StdPath, file: &syn::File) -> Vec<Violation> {
        let mut visitor = FreeFns { path, found: Vec::new() };
        visitor.visit_file(file);
        visitor.found
    }

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

struct FreeFns<'file> {
    path: &'file StdPath,
    found: Vec<Violation>,
}

impl<'ast> Visit<'ast> for FreeFns<'_> {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        let name = &function.sig.ident;
        self.found.extend(NoFreeFns::exemption(function).map_or_else(
            || Some(Violation::new(Site::new(self.path, LineNumber::from(name.span())), Finding::FreeFn { name })),
            |_| None,
        ));
        syn::visit::visit_item_fn(self, function);
    }

    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        lint_core::Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }
}

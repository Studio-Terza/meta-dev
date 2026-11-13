mod finding;

use finding::Finding;
use lint_core::{Headline, Lint};
use lint_file::FileLint;
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
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new("free functions are not allowed; make them associated functions or methods"),
            files.iter().flat_map(<Self as FileLint>::violations).collect(),
        )
    }
}

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

impl FileLint for NoFreeFns {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let mut visitor = FreeFns { file, path: file.head().path(), found: Vec::new() };
        visitor.visit_file(file.syntax());
        visitor.found
    }
}

struct FreeFns<'file> {
    file: &'file SourceFile,
    path: &'file StdPath,
    found: Vec<Violation<'file>>,
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
        lint_core::Macros::walk(self.file, invocation, |statement| self.visit_stmt(statement));
    }
}

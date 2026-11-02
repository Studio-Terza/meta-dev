mod finding;

use finding::Finding;
use lint_core::{Headline, Lint};
use syn::visit::Visit;
use syn::{Ident, ItemFn, Path};

use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::LineNumber;
use lint_layout::Name;

#[derive(Debug)]
pub struct NoFreeFns;

impl Lint for NoFreeFns {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new("free functions are not allowed; make them associated functions or methods"),
            files
                .iter()
                .flat_map(|file| {
                    Self::free_fns(file.syntax()).into_iter().map(move |name| {
                        Violation::new(
                            Site::new(file.path(), LineNumber::from(name.span())),
                            Finding::FreeFn { name: Name::from(&name) },
                        )
                    })
                })
                .collect(),
        )
    }
}

impl NoFreeFns {
    const REQUIRED_FREE: [&'static str; 4] = ["test", "proc_macro", "proc_macro_derive", "proc_macro_attribute"];

    fn free_fns(file: &syn::File) -> Vec<Ident> {
        let mut visitor = FreeFns { found: Vec::new() };
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

struct FreeFns {
    found: Vec<Ident>,
}

impl<'ast> Visit<'ast> for FreeFns {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.found.extend(NoFreeFns::exemption(function).map_or_else(|| Some(function.sig.ident.clone()), |_| None));
        syn::visit::visit_item_fn(self, function);
    }

    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        lint_core::Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }
}

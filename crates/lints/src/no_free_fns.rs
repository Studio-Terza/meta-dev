mod finding;

use crate::Lint;
use finding::Finding;
use syn::visit::Visit;
use syn::{Ident, ItemFn, Path};

use lint_core::LineNumber;
use lint_core::Name;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;

#[derive(Debug)]
pub struct NoFreeFns;

impl Lint for NoFreeFns {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "free functions are not allowed; make them associated functions or methods",
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

    fn is_exempt(function: &ItemFn) -> bool {
        function.sig.ident == "main" || function.attrs.iter().any(|attribute| Self::must_be_free(attribute.path()))
    }

    fn must_be_free(attribute: &Path) -> bool {
        attribute
            .segments
            .last()
            .is_some_and(|segment| Self::REQUIRED_FREE.iter().any(|required| segment.ident == required))
    }
}

struct FreeFns {
    found: Vec<Ident>,
}

impl<'ast> Visit<'ast> for FreeFns {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.found.extend((!NoFreeFns::is_exempt(function)).then(|| function.sig.ident.clone()));
        syn::visit::visit_item_fn(self, function);
    }

    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        lint_core::Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }
}

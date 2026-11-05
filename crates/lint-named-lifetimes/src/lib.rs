mod finding;

use finding::Finding;
use lint_core::{Headline, Lint};
use syn::visit::Visit;

use std::path::Path;

use syn::LifetimeParam;

use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::LineNumber;
use lint_layout::{Length, Name};

#[derive(Debug)]
pub struct NamedLifetimes;

impl Lint for NamedLifetimes {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new("lifetimes are named with a word, never a letter"),
            files.iter().flat_map(|file| Self::short_lifetimes(file.path(), file.syntax())).collect(),
        )
    }
}

impl NamedLifetimes {
    const MINIMUM_LENGTH: Length = Length::new(2);

    fn short_lifetimes<'file>(path: &'file Path, file: &syn::File) -> Vec<Violation<'file>> {
        let mut visitor = ShortLifetimes { path, found: Vec::new() };
        visitor.visit_file(file);
        visitor.found
    }
}

struct ShortLifetimes<'file> {
    path: &'file Path,
    found: Vec<Violation<'file>>,
}

impl<'ast> Visit<'ast> for ShortLifetimes<'_> {
    fn visit_lifetime_param(&mut self, parameter: &'ast LifetimeParam) {
        let name = &parameter.lifetime.ident;
        self.found.extend((Name::from(name).length() < NamedLifetimes::MINIMUM_LENGTH).then(|| {
            Violation::new(Site::new(self.path, LineNumber::from(name.span())), Finding::ShortLifetime { name })
        }));
    }

    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        macro_facts::Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }
}

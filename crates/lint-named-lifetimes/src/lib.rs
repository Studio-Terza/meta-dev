mod finding;

use finding::Finding;
use lint_core::Headline;
use lint_file::{FileLint, Reads};
use syn::visit::Visit;

use std::path::Path;

use syn::LifetimeParam;

use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::LineNumber;
use lint_layout::{Length, Name};

#[derive(Debug)]
pub struct NamedLifetimes;

impl NamedLifetimes {
    const MINIMUM_LENGTH: Length = Length::new(2);
}

impl FileLint for NamedLifetimes {
    const HEADLINE: Headline = Headline::new("lifetimes are named with a word, never a letter");
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let mut visitor = ShortLifetimes { file, path: file.head().path(), found: Vec::new() };
        visitor.visit_file(file.syntax());
        visitor.found
    }
}

struct ShortLifetimes<'file> {
    file: &'file SourceFile,
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
        lint_core::Macros::walk(self.file, invocation, |statement| self.visit_stmt(statement));
    }
}

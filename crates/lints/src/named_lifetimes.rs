mod finding;

use finding::Finding;
use syn::visit::Visit;

use syn::{Ident, LifetimeParam};

use lint_core::LineNumber;
use lint_core::Name;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;

#[derive(Debug)]
pub struct NamedLifetimes;

impl NamedLifetimes {
    const MINIMUM_LENGTH: usize = 2;

    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "lifetimes are named with a word, never a letter",
            files
                .iter()
                .flat_map(|file| {
                    Self::short_lifetimes(file.syntax()).into_iter().map(move |name| {
                        Violation::new(
                            Site::new(file.path(), LineNumber::from(name.span())),
                            Finding::ShortLifetime { name: Name::from(&name) },
                        )
                    })
                })
                .collect(),
        )
    }

    fn short_lifetimes(file: &syn::File) -> Vec<Ident> {
        let mut visitor = ShortLifetimes { found: Vec::new() };
        visitor.visit_file(file);
        visitor.found
    }
}

struct ShortLifetimes {
    found: Vec<Ident>,
}

impl<'ast> Visit<'ast> for ShortLifetimes {
    fn visit_lifetime_param(&mut self, parameter: &'ast LifetimeParam) {
        let name = &parameter.lifetime.ident;
        self.found.extend((Name::from(name).length() < NamedLifetimes::MINIMUM_LENGTH).then(|| name.clone()));
    }
}

mod finding;

use finding::Finding;
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Finds, Kind, Watch};

use std::path::Path;

use syn::{LifetimeParam, Macro};

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

impl SweptLint for NamedLifetimes {
    const HEADLINE: Headline = Headline::new("lifetimes are named with a word, never a letter");
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(ShortLifetimes { path: file.head().path(), found: Vec::new() })
    }
}

#[derive(Debug)]
struct ShortLifetimes<'file> {
    path: &'file Path,
    found: Vec<Violation<'file>>,
}

impl<'ast> Watch<'ast> for ShortLifetimes<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::LifetimeParam, Kind::Macro]
    }

    fn on_lifetime_param(&mut self, parameter: &'ast LifetimeParam) -> Descend {
        let name = &parameter.lifetime.ident;
        self.found.extend((Name::from(name).length() < NamedLifetimes::MINIMUM_LENGTH).then(|| {
            Violation::new(Site::new(self.path, LineNumber::from(name.span())), Finding::ShortLifetime { name })
        }));
        Descend::Over
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Expand
    }
}

impl<'file> Finds<'file> for ShortLifetimes<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        self.found
    }
}

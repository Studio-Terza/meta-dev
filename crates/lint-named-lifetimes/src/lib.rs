mod finding;

use finding::Finding;
use lint_core::Extended;
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Facts, Finds, Fold, Folding, Kind, Visit};

use std::path::Path;

use syntax::{LifetimeParam, Macro};

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
        Box::new(Folding::from(ShortLifetimes { path: AsRef::<Path>::as_ref(file.head()) }))
    }
}

#[derive(Debug)]
struct ShortLifetimes<'file> {
    path: &'file Path,
}

impl<'file> Fold for ShortLifetimes<'file> {
    type Found = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::LifetimeParam, Kind::Macro]
    }

    fn on_lifetime_param(
        &self,
        found: Self::Found,
        parameter: &LifetimeParam,
        _: &Facts<'_, LifetimeParam>,
    ) -> Visit<Self::Found> {
        let name = &parameter.lifetime.ident;
        let short = (Name::from(name).length() < NamedLifetimes::MINIMUM_LENGTH).then(|| {
            Violation::new(Site::new(self.path, LineNumber::from(name.span())), Finding::ShortLifetime { name })
        });
        Visit::new(found.extended(short), Descend::Over)
    }

    fn on_macro(&self, found: Self::Found, _: &Macro, _: &Facts<'_, Macro>) -> Visit<Self::Found> {
        Visit::new(found, Descend::Expand)
    }
}

mod finding;

use finding::Finding;
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Concludes, Contribute, Contributed, Contributing, Descend, Facts, Kind};

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

    fn watcher(file: &SourceFile) -> Contributing<impl Concludes<'_>> {
        Contributing::new(ShortLifetimes { path: AsRef::<Path>::as_ref(file.head()) })
    }
}

#[derive(Debug)]
struct ShortLifetimes<'file> {
    path: &'file Path,
}

impl<'file> Contribute for ShortLifetimes<'file> {
    type Open = ();
    type Part = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::LifetimeParam, Kind::Macro]
    }

    fn on_lifetime_param(
        &self,
        parameter: &LifetimeParam,
        _: &Facts<'_, LifetimeParam>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Vec<Violation<'file>>> {
        let name = &parameter.lifetime.ident;
        let short = (Name::from(name).length() < NamedLifetimes::MINIMUM_LENGTH).then(|| {
            Violation::new(Site::new(self.path, LineNumber::from(name.span())), Finding::ShortLifetime { name })
        });
        Contributed::new((), short.into_iter().collect(), Descend::Over)
    }

    fn on_macro(
        &self,
        _: &Macro,
        _: &Facts<'_, Macro>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Vec<Violation<'file>>> {
        Contributed::descending(Descend::Expand)
    }
}

impl<'file> Concludes<'file> for ShortLifetimes<'file> {
    fn violations(self, (): (), found: Vec<Violation<'file>>) -> Vec<Violation<'file>> {
        found
    }
}

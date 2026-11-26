use std::path::Path;
mod case;
mod flagged;
mod lines;
mod source;
mod text;

use std::fmt;
use std::path::PathBuf;

use file_ports::{Bytes, Memory};
use invariants::{invariant, invariants};
use lint_core::{Report, SourceFile, SourceText};
use lint_placement::Placement;
use newtype::Wrapping;

use crate::checks::Checks;

pub use case::{Case, Title};
pub use flagged::Flagged;
pub use lines::Lines;
pub use source::{Located, Source};
pub use text::Text;

#[derive(Debug)]
pub struct Sources(&'static [Source]);

invariants! {
    enum Invariant {
        FixtureParses => "fixture source parses",
    }
}

impl Sources {
    #[must_use]
    pub fn file(path: impl Into<PathBuf>, text: Text<'_>) -> SourceFile {
        Self::kept(SourceFile::parse(path, SourceText::new(text.into_inner().to_owned())))
    }

    pub(crate) fn kept<Parsed>(parsed: Result<Parsed, impl fmt::Display>) -> Parsed {
        match parsed {
            Ok(file) => file,
            Err(failure) => invariant!(broken Invariant::FixtureParses, failure),
        }
    }

    #[must_use]
    pub fn placed(manifests: &[Source], sources: &[Source]) -> Vec<SourceFile> {
        let files: Memory = manifests
            .iter()
            .map(|manifest| (AsRef::<Path>::as_ref(manifest).to_path_buf(), Bytes::from(manifest.text().into_inner())))
            .collect();
        sources
            .iter()
            .map(|source| {
                let placement = Placement::locate(&files, AsRef::<Path>::as_ref(source));
                let text = SourceText::new(source.text().into_inner().to_owned());
                Self::kept(SourceFile::parse_at(
                    AsRef::<Path>::as_ref(source).into(),
                    AsRef::<Path>::as_ref(source).into(),
                    placement,
                    text,
                ))
            })
            .collect()
    }

    #[must_use]
    pub fn placed_flagged<Checked: Checks>(manifests: &[Source], sources: &[Source]) -> Lines {
        Checked::flagged(&Self::placed(manifests, sources))
    }

    #[must_use]
    pub fn lines(report: &Report<'_>) -> Lines {
        Lines::new(report.violations().iter().map(|violation| violation.site().to_string()).collect())
    }

    #[must_use]
    pub fn parsed(sources: &[Source]) -> Vec<SourceFile> {
        sources.iter().map(|source| Self::file(AsRef::<Path>::as_ref(source), source.text())).collect()
    }

    #[must_use]
    pub fn flagged<Checked: Checks>(sources: &[Source]) -> Lines {
        Checked::flagged(&Self::parsed(sources))
    }

    #[must_use]
    pub fn mismatches<Checked: Checks>(cases: &[Case]) -> Lines {
        Lines::new(
            cases
                .iter()
                .filter_map(|case| {
                    let sources = Sources(case.sources());
                    let found = Self::flagged::<Checked>(sources.0);
                    (found != Lines::from(case.flagged()))
                        .then(|| format!("{} ({sources}) flagged {found}", case.name()))
                })
                .collect(),
        )
    }
}

impl fmt::Display for Sources {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|source| write!(formatter, "{} ", AsRef::<Path>::as_ref(source).display()))
    }
}

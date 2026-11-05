use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

use file_ports::Files;
use lint_core::{Readout, Report};
use lint_reader::SourceFiles;
use lint_test_strength::TestStrength;
use lint_transition_graph::{ChangeLint, Extraction, TypeSystem};

use crate::cli::Outcome;
use crate::cli::boundary::Rendered;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Revisions {
    before: PathBuf,
    after: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Usage;

impl fmt::Display for Usage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("lint strength takes exactly two roots: lint strength <before-root> <after-root>")
    }
}

impl TryFrom<&[OsString]> for Revisions {
    type Error = Usage;

    fn try_from(arguments: &[OsString]) -> Result<Revisions, Usage> {
        <&[OsString; Self::ROOTS]>::try_from(arguments)
            .map(|[before, after]| Revisions { before: PathBuf::from(before), after: PathBuf::from(after) })
            .map_err(|_| Usage)
    }
}

impl Revisions {
    const SLOT: usize = usize::MIN;
    const ROOTS: usize = 2;

    pub(crate) fn roots(&self) -> Vec<PathBuf> {
        vec![self.after.clone()]
    }

    pub(crate) fn outcome<'file>(&self, after: &'file Readout, files: &impl Files) -> Outcome<'file> {
        let rendered = SourceFiles::read(files, &self.before).map_or_else(
            |failure| vec![Rendered::failed(Self::SLOT, &failure)],
            |before| {
                Self::compared::<TestStrength>(&before, after)
                    .into_iter()
                    .map(|report| Rendered::render(Self::SLOT, &report))
                    .collect()
            },
        );
        Outcome::from_rendered(rendered, after.failures())
    }

    fn compared<'file, Change: ChangeLint>(before: &'file Readout, after: &'file Readout) -> Option<Report<'file>> {
        let was = Extraction::from(before.files());
        let now = Extraction::from(after.files());
        Change::report_change(
            &TypeSystem::built_from(before.files(), &was),
            &TypeSystem::built_from(after.files(), &now),
        )
        .dirty()
    }
}

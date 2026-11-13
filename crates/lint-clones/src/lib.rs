mod clone_sites;

use clone_sites::CloneSites;
use lint_core::{Gate, Headline, Lint, Report, SourceFile};

#[derive(Debug)]
pub struct Clones;

impl Lint for Clones {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "report only, a clone in the core allocates where a borrow could share; take a reference, a slice or &str, name a lifetime, or move the value",
            ),
            files
                .iter()
                .filter_map(SourceFile::shipped)
                .filter(|file| Gate::BOUNDARY.admits(file.head()).is_none())
                .flat_map(CloneSites::within)
                .collect(),
        )
    }
}

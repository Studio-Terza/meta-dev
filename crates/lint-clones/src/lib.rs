mod clone_sites;

use clone_sites::CloneSites;
use lint_core::{Gate, Headline, Lint, Parsed, Report, SourceFile, Violation};
use lint_file::Gleaning;

#[derive(Debug)]
pub struct Clones;

impl Clones {
    const HEADLINE: Headline = Headline::new(
        "report only, a clone in the core allocates where a borrow could share; take a reference, a slice or &str, name a lifetime, or move the value",
    );
}

impl Lint for Clones {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let findings: CloneFindings<'_> = files.iter().map(Parsed::from).map(Clones::gleaned).collect();
        Report::from(&findings)
    }
}

#[derive(Debug, Default)]
pub struct CloneFindings<'head>(Vec<Violation<'head>>);

impl<'head> FromIterator<CloneFindings<'head>> for CloneFindings<'head> {
    fn from_iter<Parts: IntoIterator<Item = CloneFindings<'head>>>(parts: Parts) -> CloneFindings<'head> {
        CloneFindings(parts.into_iter().flat_map(|part| part.0).collect())
    }
}

impl<'head> From<&CloneFindings<'head>> for Report<'head> {
    fn from(findings: &CloneFindings<'head>) -> Report<'head> {
        Report::new(Clones::HEADLINE, findings.0.clone())
    }
}

impl Gleaning for Clones {
    type Part<'head> = CloneFindings<'head>;
    type Gleaned<'head> = CloneFindings<'head>;

    fn gleaned<'head>(parsed: Parsed<'_, 'head>) -> CloneFindings<'head> {
        let path = parsed.head().path();
        let core = parsed.file().shipped().filter(|found| Gate::BOUNDARY.admits(found.head()).is_none());
        CloneFindings(
            core.map_or_default(|found| CloneSites::within(found).map(|violation| violation.moved(path)).collect()),
        )
    }
}

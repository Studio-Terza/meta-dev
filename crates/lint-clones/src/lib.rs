mod clone_sites;

use clone_sites::CloneSites;
use lint_core::{Gate, Headline, Lint, Parsed, Report, SourceFile, Violation};
use lint_file::Gleaning;
use lint_sweep::{Gathers, Sweep, Watching};

#[derive(Debug)]
pub struct Clones;

impl Clones {
    const HEADLINE: Headline = Headline::new(
        "report only, a clone in the core allocates where a borrow could share; take a reference, a slice or &str, name a lifetime, or move the value",
    );
}

impl Lint for Clones {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let findings: CloneFindings<'_> =
            files.iter().map(|file| Sweep::gathered(file, Clones::gleaner(Parsed::from(file)))).collect();
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

    type Gleaner<'head> = CloneSites<'head>;

    fn gleaner<'head>(parsed: Parsed<'_, 'head>) -> CloneSites<'head> {
        let core = parsed.file().shipped().filter(|found| Gate::BOUNDARY.admits(found.head()).is_none());
        CloneSites::new(parsed.head().path(), core.map(|_| ()))
    }
}

impl<'head> Gathers<CloneFindings<'head>> for CloneSites<'head> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> CloneFindings<'head> {
        CloneFindings(self.found())
    }
}

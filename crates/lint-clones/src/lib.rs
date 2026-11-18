use std::path::Path;
mod clone_sites;

use clone_sites::CloneSites;
use lint_core::{Gate, Gating, Headline, Lint, Parsed, Report, SourceFile, Violation};
use lint_file::Gleaning;
use lint_sweep::{Contributing, Gathers};

#[derive(Debug)]
pub struct Clones;

impl Clones {
    const HEADLINE: Headline = Headline::new(
        "report only, a clone in the core allocates where a borrow could share; take a reference, a slice or &str, name a lifetime, or move the value",
    );
}

impl Lint for Clones {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::from(&Clones::swept(files))
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

    type Gleaner<'run, 'head: 'run> = Contributing<CloneSites<'head>>;

    fn gleaner<'run, 'head: 'run>(parsed: Parsed<'run, 'head>) -> Contributing<CloneSites<'head>> {
        let core =
            <&SourceFile>::from(parsed).shipped().filter(|found| Gate::BOUNDARY.admits(found.head()) == Gating::Open);
        Contributing::new(CloneSites::new(AsRef::<Path>::as_ref(parsed.head()), core.map(|_| ())))
    }
}

impl<'head> Gathers<CloneFindings<'head>> for Contributing<CloneSites<'head>> {
    fn gathered(self, file: &SourceFile) -> CloneFindings<'head> {
        CloneFindings(self.concluded(file, |_, (), found| found))
    }
}

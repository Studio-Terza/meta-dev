mod extent;
mod finding;
mod kind;
mod occurrence;
mod scan;

use std::path::Path;

use equivalence::{Equivalence, Erased, Groups};
use lint_core::{Headline, Lint, Parsed, Report, SourceFile, Violation};
use lint_file::{ContextLint, Listed, Reads};

use finding::Finding;
use occurrence::Occurrence;
use scan::Scan;

#[derive(Debug)]
pub struct RepeatedShapes;

impl RepeatedShapes {
    const EQUIVALENCE: Equivalence = Equivalence::erasing(&[Erased::Locals, Erased::Literals]);

    fn found<'file>(
        occurrences: impl IntoIterator<Item = Occurrence<'file>>,
        equivalence: Equivalence,
    ) -> Vec<Violation<'file>> {
        Groups::grouped(occurrences)
            .outermost(|shape| shape.size())
            .listed()
            .iter()
            .filter_map(|group| Finding::violation(group, equivalence))
            .collect()
    }
}

#[derive(Debug)]
pub struct Occurrences<'head> {
    path: &'head Path,
    found: Vec<Occurrence<'head>>,
}

impl<'head> FromIterator<Occurrences<'head>> for Listed<'head> {
    fn from_iter<Parts: IntoIterator<Item = Occurrences<'head>>>(parts: Parts) -> Listed<'head> {
        let (paths, found): (Vec<&'head Path>, Vec<Vec<Occurrence<'head>>>) =
            parts.into_iter().map(|part| (part.path, part.found)).unzip();
        Listed::new(paths, RepeatedShapes::found(found.into_iter().flatten(), RepeatedShapes::EQUIVALENCE))
    }
}

impl ContextLint for RepeatedShapes {
    const HEADLINE: Headline = Headline::new(
        "a body of 40 tokens or more repeats across the workspace with its local names and literals erased; each group says whether its copies are tests, shipped code or both",
    );
    const READS: Reads = Reads::Every;

    type Part<'head> = Occurrences<'head>;
    type Context<'head> = Listed<'head>;

    fn contributed<'head>(parsed: Parsed<'_, 'head>) -> Occurrences<'head> {
        Occurrences { path: parsed.head().path(), found: Scan::of_file(parsed, Self::EQUIVALENCE) }
    }

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, listed: &Listed<'head>) -> Vec<Violation<'file>> {
        listed.found_in(file)
    }
}

#[derive(Debug)]
pub struct NormalisedShapes;

impl NormalisedShapes {
    const EQUIVALENCE: Equivalence = Equivalence::erasing(&[Erased::Names, Erased::Literals]);
}

impl Lint for NormalisedShapes {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "report only, normalised shapes: a body of 40 tokens or more repeats across the workspace once every name is erased, callees, paths, fields and types as well as locals, and literals too; groups repeated_shapes already blocks are listed again; each group says whether its copies are tests, shipped code or both",
            ),
            RepeatedShapes::found(
                files.iter().flat_map(|file| Scan::of_file(Parsed::from(file), Self::EQUIVALENCE)),
                Self::EQUIVALENCE,
            ),
        )
    }
}

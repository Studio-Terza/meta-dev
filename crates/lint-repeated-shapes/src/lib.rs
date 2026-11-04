mod extent;
mod finding;
mod kind;
mod occurrence;
mod scan;

use equivalence::{Equivalence, Erased, Groups};
use lint_core::{Headline, Lint, Report, SourceFile, Violation};

use finding::Finding;
use scan::Scan;

#[derive(Debug)]
pub struct RepeatedShapes;

impl RepeatedShapes {
    const EQUIVALENCE: Equivalence = Equivalence::erasing(&[Erased::Locals, Erased::Literals]);

    fn found(files: &[SourceFile], equivalence: Equivalence) -> Vec<Violation<'_>> {
        let occurrences = files.iter().flat_map(|file| Scan::of_file(file, equivalence));
        Groups::grouped(occurrences)
            .ranked(|shape| shape.size())
            .maximal()
            .listed()
            .iter()
            .filter_map(|group| Finding::violation(group, equivalence))
            .collect()
    }
}

impl Lint for RepeatedShapes {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "a body of 40 tokens or more repeats across the workspace with its local names and literals erased; each group says whether its copies are tests, shipped code or both",
            ),
            Self::found(files, Self::EQUIVALENCE),
        )
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
            RepeatedShapes::found(files, Self::EQUIVALENCE),
        )
    }
}

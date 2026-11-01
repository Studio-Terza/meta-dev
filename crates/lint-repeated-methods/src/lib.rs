mod shape_index;

use lint_core::Lint;
use lint_core::{Most, Report, SourceFile};

use shape_index::ShapeIndex;

#[derive(Debug)]
pub struct RepeatedMethods;

impl Lint for RepeatedMethods {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "one public method shape on many types is a missing trait; extract the trait",
            ShapeIndex::of_files(&files.iter().filter(|file| file.is_shipped()).collect::<Vec<_>>())
                .repeated(Self::TYPES_PER_SHAPE),
        )
    }
}

impl RepeatedMethods {
    const TYPES_PER_SHAPE: Most = Most::new(2);
}

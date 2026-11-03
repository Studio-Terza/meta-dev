mod shape_index;

use lint_core::Lint;
use lint_core::{Bound, Headline, Report, SourceFile};

use shape_index::ShapeIndex;

#[derive(Debug)]
pub struct RepeatedMethods;

impl Lint for RepeatedMethods {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "one public method shape on many types is repeated; use AsRef<T> for a stored field or From<&T> for a computed value, with a newtype, or a derive",
            ),
            ShapeIndex::of_files(&files.iter().filter_map(SourceFile::shipped).collect::<Vec<_>>())
                .repeated(Self::TYPES_PER_SHAPE),
        )
    }
}

impl RepeatedMethods {
    const TYPES_PER_SHAPE: Bound = Bound::Most(2);
}

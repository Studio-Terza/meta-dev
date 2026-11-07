mod queries;

use lint_core::{Headline, Lint, Report, SourceFile};
use queries::Queries;

#[derive(Debug)]
pub struct StoreQueries;

impl Lint for StoreQueries {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new(
                "every store_ports::query! writes its SQL as sqlformat formats it, starting at column 0 inside r#\"…\"#",
            ),
            files.iter().flat_map(|file| Vec::from(Queries::from(file))).collect(),
        )
    }
}

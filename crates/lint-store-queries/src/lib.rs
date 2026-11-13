mod queries;

use lint_core::{Headline, SourceFile, Violation};
use lint_file::{FileLint, Reads};
use lint_paths::Imports;
use queries::Queries;

#[derive(Debug)]
pub struct StoreQueries;

impl FileLint for StoreQueries {
    const HEADLINE: Headline = Headline::new(
        "every store_ports::query! writes its SQL as sqlformat formats it, starting at column 0 inside r#\"…\"#",
    );
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        Queries::found(file, &Imports::across([file]))
    }
}

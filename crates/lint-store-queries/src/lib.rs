mod queries;

use lint_core::{Headline, SourceFile, Violation};
use lint_file::{FileLint, Reads};
use lint_layout::{Word, words};
use lint_paths::Imports;
use queries::Queries;

#[derive(Debug)]
pub struct StoreQueries;

impl StoreQueries {
    const NAMED: [Word<'static>; 1] = words!["query"];
}

impl FileLint for StoreQueries {
    const HEADLINE: Headline = Headline::new(
        "every store_ports::query! writes its SQL as sqlformat formats it, starting at column 0 inside r#\"…\"#",
    );
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let named = file.text().mentioning(&Self::NAMED).map(|_| file);
        named.map_or_default(|queried| Queries::found(queried, &Imports::across([queried])))
    }
}

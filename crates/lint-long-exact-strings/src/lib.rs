mod assertions;

use assertions::Assertions;
use lint_core::{Headline, SourceFile, Violation};
use lint_file::{FileLint, Reads};
use lint_layout::{Word, words};

#[derive(Debug)]
pub struct LongExactStrings;

impl LongExactStrings {
    const NAMED: [Word<'static>; 2] = words!["assert", "table"];
}

impl FileLint for LongExactStrings {
    const HEADLINE: Headline = Headline::new("assertions do not pin long text");
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let asserting = file.text().mentioning(&Self::NAMED).map(|_| file);
        asserting.map_or_default(|asserted| Vec::from(Assertions::from(asserted)))
    }
}

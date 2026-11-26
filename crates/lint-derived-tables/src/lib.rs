mod tables;

use lint_core::{Headline, Parsed, SourceFile, Violation};
use lint_file::{Concluded, ContextLint, Reads, RuledLint};
use tables::{Scanning, Tables};

#[derive(Debug)]
pub struct DerivedTables;

impl ContextLint for DerivedTables {
    const HEADLINE: Headline =
        Headline::new("an enum's variant list and spellings are derived, never kept by hand in a table or a match");
    const READS: Reads = Reads::Every;

    type Part<'head> = Tables<'head>;
    type Context<'head> = Concluded<'head>;

    type Gathering<'head> = Scanning<'head>;

    fn gathering<'head>(parsed: Parsed<'_, 'head>) -> Scanning<'head> {
        Scanning::reading(parsed)
    }
}

impl RuledLint for DerivedTables {
    fn ruled<'file, 'head: 'file>(file: &'file SourceFile, concluded: &Concluded<'head>) -> Vec<Violation<'file>> {
        concluded.found_in(file)
    }
}

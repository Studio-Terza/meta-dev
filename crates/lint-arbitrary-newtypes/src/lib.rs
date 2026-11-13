mod ungenerated;

use lint_core::{Headline, SourceFile, Violation};
use lint_file::{FileLint, Reads};
use ungenerated::Ungenerated;

#[derive(Debug)]
pub struct ArbitraryNewtypes;

impl FileLint for ArbitraryNewtypes {
    const HEADLINE: Headline = Headline::new("a newtype that owns its value derives Arbitrary");
    const READS: Reads = Reads::Shipped;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        Vec::from(Ungenerated::from(file))
    }
}

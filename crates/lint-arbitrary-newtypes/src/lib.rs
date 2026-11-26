mod ungenerated;

use lint_core::{Headline, SourceFile};
use lint_file::{Reads, SweptLint};
use lint_sweep::{Concludes, Contributing};
use ungenerated::Ungenerated;

#[derive(Debug)]
pub struct ArbitraryNewtypes;

impl SweptLint for ArbitraryNewtypes {
    const HEADLINE: Headline = Headline::new("a newtype that owns its value derives Arbitrary");
    const READS: Reads = Reads::Shipped;

    fn watcher(file: &SourceFile) -> Contributing<impl Concludes<'_>> {
        Contributing::new(Ungenerated::from(file))
    }
}

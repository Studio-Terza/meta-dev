mod ungenerated;

use lint_core::{Headline, SourceFile};
use lint_file::{Reads, SweptLint};
use lint_sweep::Finds;
use ungenerated::Ungenerated;

#[derive(Debug)]
pub struct ArbitraryNewtypes;

impl SweptLint for ArbitraryNewtypes {
    const HEADLINE: Headline = Headline::new("a newtype that owns its value derives Arbitrary");
    const READS: Reads = Reads::Shipped;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(Ungenerated::from(file))
    }
}

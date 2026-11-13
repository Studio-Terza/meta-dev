mod exits;

use exits::Exits;
use lint_core::{Headline, SourceFile};
use lint_file::{Reads, SweptLint};
use lint_sweep::Finds;

#[derive(Debug)]
pub struct NoEarlyExits;

impl SweptLint for NoEarlyExits {
    const HEADLINE: Headline =
        Headline::new("a body leaves through its tail value or ?, never through return, break or continue");
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(Exits::from(file))
    }
}

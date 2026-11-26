mod returns;

use lint_core::{Headline, SourceFile};
use lint_file::{Reads, SweptLint};
use lint_sweep::{Concludes, Contributing};
use returns::Returns;

#[derive(Debug)]
pub struct BoolMethods;

impl SweptLint for BoolMethods {
    const HEADLINE: Headline =
        Headline::new("a function returning bool forgets what it found and makes the caller branch on a flag");
    const READS: Reads = Reads::Shipped;

    fn watcher(file: &SourceFile) -> Contributing<impl Concludes<'_>> {
        Contributing::new(Returns::from(file))
    }
}

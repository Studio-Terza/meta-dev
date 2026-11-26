use std::path::Path;
mod assertions;

use assertions::Assertions;
use lint_core::{Headline, SourceFile};
use lint_file::{Reads, SweptLint};
use lint_layout::{Word, words};
use lint_sweep::{Concludes, Contributing};

#[derive(Debug)]
pub struct LongExactStrings;

impl LongExactStrings {
    const NAMED: [Word<'static>; 2] = words!["assert", "table"];
}

impl SweptLint for LongExactStrings {
    const HEADLINE: Headline = Headline::new("assertions do not pin long text");
    const READS: Reads = Reads::Every;

    fn watcher(file: &SourceFile) -> Contributing<impl Concludes<'_>> {
        let asserting = file.text().mentioning(&Self::NAMED).map(|_| AsRef::<Path>::as_ref(file.head()));
        Contributing::new(Assertions::new(file, asserting))
    }
}

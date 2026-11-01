mod finding;
mod pair;
mod paired_methods;

use finding::Finding;
use lint_core::{Lint, Report, SourceFile};
use pair::Pair;
use paired_methods::PairedMethods;

#[derive(Debug)]
pub struct PairedApis;

impl Lint for PairedApis {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "report only, a paired API leaves the ending to every caller; write a guard whose Drop does the ending",
            files.iter().filter(|file| file.is_shipped()).flat_map(PairedMethods::of_file).collect(),
        )
    }
}

mod declared;
mod finding;
mod owner;
mod pair;
mod paired_methods;
mod pairs;

use finding::Finding;
use lint_core::{Headline, Lint, Report, SourceFile};
use pair::Pair;
use paired_methods::PairedMethods;
use pairs::Pairs;

#[derive(Debug)]
pub struct PairedApis;

impl Lint for PairedApis {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let declared = PairedMethods::of_files(files);
        Report::new(
            Headline::new(
                "report only, a paired API leaves the ending to every caller; write a guard whose Drop does the ending",
            ),
            Pairs::listed(&declared),
        )
    }
}

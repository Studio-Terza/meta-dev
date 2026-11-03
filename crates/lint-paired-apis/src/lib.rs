mod declared;
mod owner;
mod paired_methods;
mod pairs;

use lint_core::{Headline, Lint, Report, SourceFile};
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

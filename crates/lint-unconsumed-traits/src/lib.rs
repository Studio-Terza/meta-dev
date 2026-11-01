mod bounds;
mod declaration;
mod declared;
mod finding;

use std::collections::BTreeSet;

use lint_core::{Headline, Lint, Report, Site, SourceFile, Violation};
use lint_layout::Name;

use bounds::Bounds;
use declared::Declared;
use finding::Finding;

#[derive(Debug)]
pub struct UnconsumedTraits;

impl Lint for UnconsumedTraits {
    fn report(files: &[SourceFile]) -> Report {
        let shipped: Vec<&SourceFile> = files.iter().filter(|file| file.is_shipped()).collect();
        let consumers: BTreeSet<Name> = shipped.iter().flat_map(|file| Bounds::of_file(file)).collect();
        Report::new(
            Headline::new(
                "report only, a local trait has methods but no generic consumer, no bound, impl Trait, dyn Trait or provided method",
            ),
            shipped
                .iter()
                .flat_map(|file| Declared::of_file(file))
                .filter(|declaration| !declaration.is_consumed_by(&consumers))
                .map(|declaration| {
                    Violation::new(AsRef::<Site>::as_ref(&declaration).clone(), Finding::from(&declaration))
                })
                .collect(),
        )
    }
}

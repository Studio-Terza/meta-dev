mod consumers;
mod declaration;
mod declared;
mod finding;

use lint_core::{Headline, Lint, Report, Site, SourceFile, Violation};

use consumers::Consumers;
use declared::Declared;
use finding::Finding;

#[derive(Debug)]
pub struct UnconsumedTraits;

impl Lint for UnconsumedTraits {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let shipped: Vec<&SourceFile> = files.iter().filter_map(SourceFile::shipped).collect();
        let consumers = Consumers::of_files(shipped.iter().copied());
        Report::new(
            Headline::new(
                "a local trait with methods has a consumer in shipped code: a bound (generic, where clause, supertrait), impl Trait, dyn Trait, or a call to one of its provided methods",
            ),
            shipped
                .iter()
                .flat_map(|file| Declared::of_file(file))
                .filter_map(|declaration| declaration.unconsumed(&consumers))
                .map(|declaration| Violation::new(*AsRef::<Site>::as_ref(&declaration), Finding::from(&declaration)))
                .collect(),
        )
    }
}

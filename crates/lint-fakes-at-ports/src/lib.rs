mod doubles;
mod port_traits;

use doubles::Doubles;
pub use doubles::Kind;
use lint_core::{Headline, Lint, Report, SourceFile};
use lint_paths::Imports;
pub use port_traits::PortTraits;

#[derive(Debug)]
pub struct FakesAtPorts;

impl Lint for FakesAtPorts {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let tested: Vec<&SourceFile> = files.iter().filter_map(Doubles::tested).collect();
        let imports = Imports::across(tested.iter().copied());
        let ports = PortTraits::declared(files);
        let doubles = tested.iter().fold(Doubles::default(), |mut doubles, file| {
            doubles.scan(file, &imports, &ports);
            doubles
        });
        Report::new(Headline::new("test doubles stand in for ports"), doubles.violations())
    }
}

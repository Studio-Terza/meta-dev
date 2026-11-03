mod holders;
mod judge;
mod reader;
mod spelling;

use lint_core::{Headline, Lint, Report, SourceFile};
use lint_fakes_at_ports::PortTraits;
use lint_paths::Imports;

use holders::Holders;
use judge::Judge;
use reader::Reader;

#[derive(Debug)]
pub struct PortBatching;

impl Lint for PortBatching {
    fn report(files: &[SourceFile]) -> Report<'_> {
        let shipped: Vec<&SourceFile> = files.iter().filter_map(SourceFile::shipped).collect();
        let imports = Imports::across(shipped.iter().copied());
        let ports = PortTraits::declared(files);
        let judge = Judge::new(&imports, &ports);
        let holders = Holders::declared(&shipped, &judge);
        Report::new(
            Headline::new("ports are called once per batch, not once per element"),
            shipped.iter().flat_map(|file| Reader::violations(file, &judge, &holders)).collect(),
        )
    }
}

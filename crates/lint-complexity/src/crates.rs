mod distribution;
mod function;
mod functions;

use std::collections::BTreeMap;

use lint_core::{SourceFile, Violation};
use lint_placement::CrateName;

use distribution::Distribution;
use function::Function;
use functions::Functions;

#[derive(Debug)]
pub(crate) struct Crates<'file>(BTreeMap<CrateName, Vec<Function<'file>>>);

impl<'file> Crates<'file> {
    pub(crate) fn grouped(files: &'file [SourceFile]) -> Crates<'file> {
        Crates(files.iter().filter_map(SourceFile::shipped).fold(BTreeMap::new(), |mut crates, file| {
            let name = CrateName::from(file.head().placement().package().root());
            crates.entry(name).or_default().extend(Functions::of_file(file));
            crates
        }))
    }

    pub(crate) fn violations(&self) -> Vec<Violation<'file>> {
        self.0
            .iter()
            .filter_map(|(name, functions)| Distribution::listing(name, functions))
            .map(|found| found.violation())
            .collect()
    }
}

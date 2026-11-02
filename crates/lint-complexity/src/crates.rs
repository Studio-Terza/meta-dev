mod distribution;
mod function;
mod functions;
mod score;

use std::collections::BTreeMap;

use lint_core::{SourceFile, Violation};
use lint_placement::CrateName;

use distribution::Distribution;
use function::Function;
use functions::Functions;

#[derive(Debug)]
pub(crate) struct Crates(BTreeMap<CrateName, Vec<Function>>);

impl Crates {
    pub(crate) fn grouped(files: &[SourceFile]) -> Crates {
        Crates(files.iter().filter(|file| file.is_shipped()).fold(BTreeMap::new(), |mut crates, file| {
            let name = CrateName::from(file.placement().package().root());
            crates.entry(name).or_default().extend(Functions::of_file(file));
            crates
        }))
    }

    pub(crate) fn violations(&self) -> Vec<Violation> {
        self.0
            .iter()
            .filter_map(|(name, functions)| Distribution::listing(name, functions))
            .map(|found| found.violation())
            .collect()
    }
}

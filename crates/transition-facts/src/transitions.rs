mod collector;

use lint_core::SourceFile;

use crate::transition::Transition;
use collector::Collector;

#[derive(Clone, Debug, Default)]
pub struct Transitions(Vec<Transition>);

impl Transitions {
    #[must_use]
    pub fn of_files(files: &[SourceFile]) -> Transitions {
        Transitions(files.iter().filter_map(SourceFile::shipped).flat_map(Collector::collected).collect())
    }

    #[must_use]
    pub fn all(&self) -> &[Transition] {
        &self.0
    }
}

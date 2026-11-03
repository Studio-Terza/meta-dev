mod collector;

use lint_core::SourceFile;

use crate::transition::Transition;
use collector::Collector;

#[derive(Clone, Debug, Default)]
pub struct Transitions<'file>(Vec<Transition<'file>>);

impl<'file> Transitions<'file> {
    #[must_use]
    pub fn of_files(files: &'file [SourceFile]) -> Transitions<'file> {
        Transitions(files.iter().filter_map(SourceFile::shipped).flat_map(Collector::collected).collect())
    }

    #[must_use]
    pub fn all(&self) -> &[Transition<'file>] {
        &self.0
    }
}

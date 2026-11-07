mod collector;

use lint_core::SourceFile;

use crate::context::Context;
use crate::transition::Transition;
use collector::Collector;

#[derive(Debug, Default)]
pub struct Transitions<'file>(Vec<Context<'file>>);

impl<'file> Transitions<'file> {
    #[must_use]
    pub fn of_files(files: &'file [SourceFile]) -> Self {
        Transitions(files.iter().filter_map(SourceFile::shipped).flat_map(Collector::collected).collect())
    }

    #[must_use]
    pub fn contexts(&self) -> &[Context<'file>] {
        &self.0
    }

    pub fn all(&self) -> impl Iterator<Item = Transition<'_, 'file>> {
        self.0
            .iter()
            .flat_map(|context| context.functions().iter().map(move |function| Transition::new(context, function)))
    }
}

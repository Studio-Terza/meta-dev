mod collector;

use lint_core::{Parsed, SourceFile};

use crate::context::Context;
use crate::transition::Transition;
use collector::Collector;

#[derive(Debug, Default)]
pub struct Transitions<'file>(Vec<Context<'file>>);

impl<'file> Transitions<'file> {
    #[must_use]
    pub fn of_files(files: &'file [SourceFile]) -> Transitions<'file> {
        files.iter().map(Parsed::from).map(Transitions::from).collect()
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

impl<'head> From<Parsed<'_, 'head>> for Transitions<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> Transitions<'head> {
        let head = parsed.head();
        Transitions(head.shipped().map_or_default(|_| Collector::collected(parsed.file(), head)))
    }
}

impl<'file> FromIterator<Transitions<'file>> for Transitions<'file> {
    fn from_iter<Parts: IntoIterator<Item = Transitions<'file>>>(parts: Parts) -> Transitions<'file> {
        Transitions(parts.into_iter().flat_map(|part| part.0).collect())
    }
}

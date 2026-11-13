use graph::{ItemPath, Model, Named, Output, Primitive, Resolved, Sited, TypeRef};
use lint_layout::{Word, words};

use crate::error_types::ErrorTypes;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Narrowed {
    Flag,
    Check,
    Problem,
}

#[derive(Clone, Copy, Debug)]
struct StdShape {
    path: &'static [Word<'static>],
    narrowed: Narrowed,
}

impl StdShape {
    const ROWS: [StdShape; 2] = [
        StdShape { path: &words!["std", "result", "Result"], narrowed: Narrowed::Check },
        StdShape { path: &words!["std", "option", "Option"], narrowed: Narrowed::Problem },
    ];

    fn path(self) -> ItemPath {
        self.path.iter().copied().collect()
    }
}

impl Narrowed {
    pub(crate) fn returned(model: &Model, errors: &ErrorTypes, output: &Output) -> Option<Narrowed> {
        let expanded = model.expanded(output.written()?);
        Self::named(errors, expanded.named()?)
    }

    pub(crate) fn shown(self) -> Word<'static> {
        match self {
            Narrowed::Flag => Word::new("bool"),
            Narrowed::Check => Word::new("Result<(), E>"),
            Narrowed::Problem => Word::new("Option<E>"),
        }
    }

    fn named(errors: &ErrorTypes, named: &Named<Sited>) -> Option<Narrowed> {
        let path: &Resolved = named.as_ref();
        let flag = path.primitive().into_iter().find(|primitive| *primitive == Primitive::Bool).map(|_| Narrowed::Flag);
        flag.or_else(|| {
            let std = path.std()?;
            StdShape::ROWS.iter().find(|row| row.path() == *std)?.narrowed.argued(errors, named.arguments())
        })
    }

    fn argued(self, errors: &ErrorTypes, arguments: &[TypeRef]) -> Option<Narrowed> {
        match self {
            Narrowed::Check => arguments.first().and_then(TypeRef::unit).map(|_| Narrowed::Check),
            Narrowed::Problem => arguments.first().and_then(|error| errors.holding(error)).map(|_| Narrowed::Problem),
            Narrowed::Flag => None,
        }
    }
}

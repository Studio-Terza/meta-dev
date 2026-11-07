use code_model::{ItemPath, Model, Named, Output, Primitive, Resolved, Sited, TypeRef};
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
    const ROWS: [Self; 2] = [
        Self { path: &words!["std", "result", "Result"], narrowed: Narrowed::Check },
        Self { path: &words!["std", "option", "Option"], narrowed: Narrowed::Problem },
    ];

    fn path(self) -> ItemPath {
        self.path.iter().copied().collect()
    }
}

impl Narrowed {
    pub(crate) fn returned(model: &Model, errors: &ErrorTypes, output: &Output) -> Option<Self> {
        let expanded = model.expanded(output.written()?);
        Self::named(errors, expanded.named()?)
    }

    pub(crate) fn shown(self) -> Word<'static> {
        match self {
            Self::Flag => Word::new("bool"),
            Self::Check => Word::new("Result<(), E>"),
            Self::Problem => Word::new("Option<E>"),
        }
    }

    fn named(errors: &ErrorTypes, named: &Named<Sited>) -> Option<Self> {
        let path: &Resolved = named.as_ref();
        let flag = path.primitive().into_iter().find(|primitive| *primitive == Primitive::Bool).map(|_| Self::Flag);
        flag.or_else(|| {
            let std = path.std()?;
            StdShape::ROWS.iter().find(|row| row.path() == *std)?.narrowed.argued(errors, named.arguments())
        })
    }

    fn argued(self, errors: &ErrorTypes, arguments: &[TypeRef]) -> Option<Self> {
        match self {
            Self::Check => arguments.first().and_then(TypeRef::unit).map(|_| Self::Check),
            Self::Problem => arguments.first().and_then(|error| errors.holding(error)).map(|_| Self::Problem),
            Self::Flag => None,
        }
    }
}

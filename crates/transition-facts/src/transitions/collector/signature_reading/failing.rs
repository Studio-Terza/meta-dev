mod expanding;

use syn::{GenericArgument, Path, PathArguments, ReturnType, Type};

use lint_layout::{Name, Whole, Word, words};

use crate::transitions::collector::origins::Origins;
use expanding::Expanding;

#[derive(Debug)]
pub(super) struct Failing;

impl Failing {
    const RESULT: [Word<'static>; 1] = words!["Result"];
    const ERROR: usize = 1;

    pub(super) fn of_output<'found>(output: &'found ReturnType, origins: &'found Origins<'_>) -> Option<&'found Type> {
        match output {
            ReturnType::Default => None,
            ReturnType::Type(_, returned) => Self::peeled(returned, origins, &Expanding::default()),
        }
    }

    fn peeled<'found>(
        found: &'found Type,
        origins: &'found Origins<'_>,
        expanding: &Expanding,
    ) -> Option<&'found Type> {
        match found {
            Type::Reference(reference) => Self::peeled(&reference.elem, origins, expanding),
            Type::Paren(paren) => Self::peeled(&paren.elem, origins, expanding),
            Type::Path(path) => Self::error_of(&path.path, origins, expanding),
            _ => None,
        }
    }

    fn error_of<'found>(
        path: &'found Path,
        origins: &'found Origins<'_>,
        expanding: &Expanding,
    ) -> Option<&'found Type> {
        let result = path
            .segments
            .last()
            .filter(|segment| Name::from(&segment.ident).matches::<Whole>(&Self::RESULT).is_some())
            .map(|segment| &segment.arguments);
        result
            .into_iter()
            .flat_map(Self::arguments)
            .filter_map(Self::typed)
            .nth(Self::ERROR)
            .or_else(|| expanding.through(path, origins, |target, within| Self::peeled(target, origins, within)))
    }

    fn arguments(arguments: &PathArguments) -> Vec<&GenericArgument> {
        match arguments {
            PathArguments::AngleBracketed(bracketed) => bracketed.args.iter().collect(),
            PathArguments::None | PathArguments::Parenthesized(_) => Vec::new(),
        }
    }

    fn typed(argument: &GenericArgument) -> Option<&Type> {
        match argument {
            GenericArgument::Type(inner) => Some(inner),
            _ => None,
        }
    }
}

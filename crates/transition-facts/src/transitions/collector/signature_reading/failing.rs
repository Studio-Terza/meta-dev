use syn::{GenericArgument, PathArguments, PathSegment, ReturnType, Type};

use lint_layout::{Name, Whole, Word, words};

use crate::transitions::collector::origins::Origins;

#[derive(Debug)]
pub(super) struct Failing;

impl Failing {
    const RESULT: [Word<'static>; 1] = words!["Result"];
    const ERROR: usize = 1;

    pub(super) fn of_output<'found>(output: &'found ReturnType, origins: &'found Origins<'_>) -> Option<&'found Type> {
        match output {
            ReturnType::Default => None,
            ReturnType::Type(_, returned) => Self::peeled(returned, origins),
        }
    }

    fn peeled<'found>(found: &'found Type, origins: &'found Origins<'_>) -> Option<&'found Type> {
        match found {
            Type::Reference(reference) => Self::peeled(&reference.elem, origins),
            Type::Paren(paren) => Self::peeled(&paren.elem, origins),
            Type::Path(path) => path.path.segments.last().and_then(|segment| Self::error_of(segment, origins)),
            _ => None,
        }
    }

    fn error_of<'found>(segment: &'found PathSegment, origins: &'found Origins<'_>) -> Option<&'found Type> {
        let result = Name::from(&segment.ident).matches::<Whole>(&Self::RESULT).map(|_| &segment.arguments);
        result
            .into_iter()
            .flat_map(Self::arguments)
            .filter_map(Self::typed)
            .nth(Self::ERROR)
            .or_else(|| origins.aliased(&Name::from(&segment.ident)).and_then(|target| Self::peeled(target, origins)))
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

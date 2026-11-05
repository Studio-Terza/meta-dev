use syn::{GenericArgument, PathArguments, PathSegment, ReturnType, Type};

use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub(super) struct Failing;

impl Failing {
    const RESULT: [Word<'static>; 1] = words!["Result"];
    const ERROR: usize = 1;

    pub(super) fn of_output(output: &ReturnType) -> Option<&Type> {
        match output {
            ReturnType::Default => None,
            ReturnType::Type(_, returned) => Self::peeled(returned),
        }
    }

    fn peeled(found: &Type) -> Option<&Type> {
        match found {
            Type::Reference(reference) => Self::peeled(&reference.elem),
            Type::Paren(paren) => Self::peeled(&paren.elem),
            Type::Path(path) => path.path.segments.last().and_then(Self::error_of),
            _ => None,
        }
    }

    fn error_of(segment: &PathSegment) -> Option<&Type> {
        let result = Name::from(&segment.ident).matches::<Whole>(&Self::RESULT).map(|_| &segment.arguments);
        result.into_iter().flat_map(Self::arguments).filter_map(Self::typed).nth(Self::ERROR)
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

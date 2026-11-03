use syn::{FnArg, GenericArgument, PatType, PathArguments, PathSegment, ReturnType, Type};

use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct Spelled;

impl Spelled {
    pub(crate) fn last_of(found: &Type) -> Option<Name> {
        match found {
            Type::Path(path) => path.path.segments.last().map(|segment| Name::from(&segment.ident)),
            Type::Reference(reference) => Self::last_of(&reference.elem),
            Type::Paren(paren) => Self::last_of(&paren.elem),
            _ => None,
        }
    }

    pub(crate) fn first_argument_of(segment: &PathSegment) -> Option<Name> {
        match &segment.arguments {
            PathArguments::AngleBracketed(arguments) => arguments.args.iter().find_map(Self::of_argument),
            _ => None,
        }
    }

    pub(crate) fn returned_by(output: &ReturnType) -> Option<&Type> {
        match output {
            ReturnType::Type(_, found) => Some(found),
            ReturnType::Default => None,
        }
    }

    pub(crate) fn typed(input: &FnArg) -> Option<&PatType> {
        match input {
            FnArg::Typed(typed) => Some(typed),
            FnArg::Receiver(_) => None,
        }
    }

    fn of_argument(argument: &GenericArgument) -> Option<Name> {
        match argument {
            GenericArgument::Type(found) => Self::last_of(found),
            _ => None,
        }
    }
}

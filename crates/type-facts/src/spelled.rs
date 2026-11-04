use syn::{FnArg, GenericArgument, PatType, PathArguments, PathSegment, ReturnType, Type, TypeParamBound};

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

    pub(crate) fn bound_of(bound: &TypeParamBound) -> Option<Name> {
        match bound {
            TypeParamBound::Trait(found) => found.path.segments.last().map(|last| Name::from(&last.ident)),
            _ => None,
        }
    }

    pub(crate) fn qualifier_of(found: &Type) -> Vec<Name> {
        match found {
            Type::Path(path) => Self::qualifier(&path.path),
            Type::Reference(reference) => Self::qualifier_of(&reference.elem),
            Type::Paren(paren) => Self::qualifier_of(&paren.elem),
            Type::Group(group) => Self::qualifier_of(&group.elem),
            _ => Vec::new(),
        }
    }

    pub(crate) fn qualifier(path: &syn::Path) -> Vec<Name> {
        path.segments.iter().rev().skip(1).rev().map(|step| Name::from(&step.ident)).collect()
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

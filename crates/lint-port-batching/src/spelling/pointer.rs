use syn::{AngleBracketedGenericArguments, GenericArgument, PathArguments, PathSegment, Type};

use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub struct Pointer;

impl Pointer {
    const POINTERS: [Word<'static>; 3] = words!["Box", "Rc", "Arc"];

    pub fn pointee(segment: &PathSegment) -> Option<&Type> {
        let pointer = Some(segment).filter(|last| Name::from(&last.ident).matches::<Whole>(&Self::POINTERS).is_some());
        pointer
            .and_then(|last| Self::angled(&last.arguments))
            .into_iter()
            .flat_map(|angled| &angled.args)
            .find_map(Self::typed)
    }

    fn angled(arguments: &PathArguments) -> Option<&AngleBracketedGenericArguments> {
        match arguments {
            PathArguments::AngleBracketed(angled) => Some(angled),
            _ => None,
        }
    }

    fn typed(argument: &GenericArgument) -> Option<&Type> {
        match argument {
            GenericArgument::Type(inner) => Some(inner),
            _ => None,
        }
    }
}

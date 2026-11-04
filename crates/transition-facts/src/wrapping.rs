use syn::{GenericArgument, PathArguments, PathSegment, ReturnType, Type};

use lint_layout::{Name, Whole, Word, words};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Wrapping {
    Plain,
    Optional,
    Fallible,
    Many,
}

impl Wrapping {
    const TRANSPARENT: [Word<'static>; 8] = words!["Box", "Rc", "Arc", "Cow", "RefCell", "Cell", "Mutex", "Reverse"];
    const COLLECTIONS: [Word<'static>; 4] = words!["Vec", "VecDeque", "BTreeSet", "HashSet"];
    const OPTION: [Word<'static>; 1] = words!["Option"];
    const RESULT: [Word<'static>; 1] = words!["Result"];

    pub(crate) fn around(self, inner: Wrapping) -> Wrapping {
        Some(inner).filter(|inner| *inner == Wrapping::Many).unwrap_or(self)
    }

    pub(crate) fn returned(output: &ReturnType) -> Wrapping {
        match output {
            ReturnType::Default => Wrapping::Plain,
            ReturnType::Type(_, returned) => Self::peeled(returned),
        }
    }

    fn peeled(found: &Type) -> Wrapping {
        match found {
            Type::Reference(reference) => Self::peeled(&reference.elem),
            Type::Paren(paren) => Self::peeled(&paren.elem),
            Type::Group(group) => Self::peeled(&group.elem),
            Type::Path(path) => path.path.segments.last().map_or(Wrapping::Plain, Self::named),
            _ => Wrapping::Plain,
        }
    }

    fn named(segment: &PathSegment) -> Wrapping {
        let name = Name::from(&segment.ident);
        let is = |words: &[Word<'static>]| name.matches::<Whole>(words).is_some();
        [(&Self::OPTION[..], Wrapping::Optional), (&Self::RESULT[..], Wrapping::Fallible)]
            .into_iter()
            .find(|(words, _)| is(words))
            .map(|(_, wrapping)| wrapping.around(Self::inside(&segment.arguments)))
            .or_else(|| is(&Self::COLLECTIONS).then_some(Wrapping::Many))
            .or_else(|| is(&Self::TRANSPARENT).then(|| Self::inside(&segment.arguments)))
            .unwrap_or(Wrapping::Plain)
    }

    fn inside(arguments: &PathArguments) -> Wrapping {
        match arguments {
            PathArguments::AngleBracketed(bracketed) => {
                bracketed.args.iter().find_map(Self::argued).unwrap_or(Wrapping::Plain)
            }
            PathArguments::None | PathArguments::Parenthesized(_) => Wrapping::Plain,
        }
    }

    fn argued(argument: &GenericArgument) -> Option<Wrapping> {
        match argument {
            GenericArgument::Type(inner) => Some(Self::peeled(inner)),
            _ => None,
        }
    }
}

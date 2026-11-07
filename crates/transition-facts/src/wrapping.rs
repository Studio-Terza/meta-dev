mod bounds;

use syn::{GenericArgument, PathArguments, PathSegment, ReturnType, Type, TypePath};

use crate::associated::Associated;
pub(crate) use bounds::Bounds;
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
    pub(crate) const ITERATORS: [Word<'static>; 4] =
        words!["Iterator", "IntoIterator", "DoubleEndedIterator", "ExactSizeIterator"];
    const OPTION: [Word<'static>; 1] = words!["Option"];
    const RESULT: [Word<'static>; 1] = words!["Result"];

    pub(crate) fn around(self, inner: Self) -> Self {
        Some(inner).filter(|inner| *inner == Self::Many).unwrap_or(self)
    }

    pub(crate) fn returned(
        output: &ReturnType,
        associated: Option<Self>,
        generic: &impl Fn(&Name) -> Option<Self>,
    ) -> Self {
        match output {
            ReturnType::Default => Self::Plain,
            ReturnType::Type(_, returned) => {
                Associated::of_type(returned).and(associated).unwrap_or_else(|| Self::peeled(returned, generic))
            }
        }
    }

    pub(crate) fn peeled(found: &Type, generic: &impl Fn(&Name) -> Option<Self>) -> Self {
        match found {
            Type::Reference(reference) => Self::peeled(&reference.elem, generic),
            Type::Paren(paren) => Self::peeled(&paren.elem, generic),
            Type::Array(_) | Type::Slice(_) => Self::Many,
            Type::Path(path) => Self::pathed(path, generic),
            Type::ImplTrait(found) => Bounds::new(&found.bounds).wrapping(),
            Type::TraitObject(found) => Bounds::new(&found.bounds).wrapping(),
            _ => Self::Plain,
        }
    }

    fn pathed(path: &TypePath, generic: &impl Fn(&Name) -> Option<Self>) -> Self {
        path.path
            .get_ident()
            .and_then(|ident| generic(&Name::from(ident)))
            .or_else(|| path.path.segments.last().map(|last| Self::named(last, generic)))
            .unwrap_or(Self::Plain)
    }

    fn named(segment: &PathSegment, generic: &impl Fn(&Name) -> Option<Self>) -> Self {
        let name = Name::from(&segment.ident);
        let is = |words: &[Word<'static>]| name.matches::<Whole>(words).is_some();
        [(&Self::OPTION[..], Self::Optional), (&Self::RESULT[..], Self::Fallible)]
            .into_iter()
            .find(|(words, _)| is(words))
            .map(|(_, wrapping)| wrapping.around(Self::inside(&segment.arguments, generic)))
            .or_else(|| is(&Self::COLLECTIONS).then_some(Self::Many))
            .or_else(|| is(&Self::TRANSPARENT).then(|| Self::inside(&segment.arguments, generic)))
            .unwrap_or(Self::Plain)
    }

    fn inside(arguments: &PathArguments, generic: &impl Fn(&Name) -> Option<Self>) -> Self {
        match arguments {
            PathArguments::AngleBracketed(bracketed) => {
                bracketed.args.iter().find_map(|argument| Self::argued(argument, generic)).unwrap_or(Self::Plain)
            }
            PathArguments::None | PathArguments::Parenthesized(_) => Self::Plain,
        }
    }

    fn argued(argument: &GenericArgument, generic: &impl Fn(&Name) -> Option<Self>) -> Option<Self> {
        match argument {
            GenericArgument::Type(inner) => Some(Self::peeled(inner, generic)),
            _ => None,
        }
    }
}

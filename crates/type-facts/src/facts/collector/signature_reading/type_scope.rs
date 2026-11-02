use std::collections::BTreeSet;

use syn::{GenericArgument, PathArguments, PathSegment, Type, TypeParamBound};

use lint_layout::{Name, Whole, Word, words};

use crate::endpoint::Endpoint;

#[derive(Debug)]
pub(super) struct TypeScope<'owner> {
    owner: Option<&'owner Name>,
    generics: BTreeSet<Name>,
}

impl<'owner> TypeScope<'owner> {
    const WRAPPERS: [Word<'static>; 14] = words![
        "Option", "Result", "Vec", "Box", "Rc", "Arc", "Cow", "VecDeque", "BTreeSet", "HashSet", "RefCell", "Cell",
        "Mutex", "Reverse"
    ];
    const PRIMITIVES: [Word<'static>; 18] = words![
        "bool", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64",
        "char", "str", "String"
    ];
    const ITEM: [Word<'static>; 1] = words!["Item"];
    const SELF: [Word<'static>; 1] = words!["Self"];

    pub(super) fn new(owner: Option<&'owner Name>, generics: BTreeSet<Name>) -> TypeScope<'owner> {
        TypeScope { owner, generics }
    }

    pub(super) fn receiver(&self) -> Endpoint {
        self.owner.map_or(Endpoint::Opaque, |owner| Endpoint::Named(owner.clone()))
    }

    pub(super) fn endpoint(&self, found: &Type) -> Endpoint {
        match found {
            Type::Reference(reference) => self.endpoint(&reference.elem),
            Type::Paren(paren) => self.endpoint(&paren.elem),
            Type::Slice(slice) => self.endpoint(&slice.elem),
            Type::Array(array) => self.endpoint(&array.elem),
            Type::Tuple(tuple) if tuple.elems.is_empty() => Endpoint::Unit,
            Type::Path(path) => path.path.segments.last().map_or(Endpoint::Opaque, |last| self.named(last)),
            Type::ImplTrait(bounds) => {
                bounds.bounds.iter().find_map(|bound| self.item_of(bound)).unwrap_or(Endpoint::Opaque)
            }
            _ => Endpoint::Opaque,
        }
    }

    fn named(&self, segment: &PathSegment) -> Endpoint {
        let name = Name::from(&segment.ident);
        match name {
            _ if name.matches::<Whole>(&Self::WRAPPERS).is_some() => {
                self.argument(&segment.arguments).unwrap_or(Endpoint::Unit)
            }
            _ if name.matches::<Whole>(&Self::PRIMITIVES).is_some() => Endpoint::Primitive(name),
            _ if name.matches::<Whole>(&Self::SELF).is_some() => self.receiver(),
            _ if self.generics.contains(&name) => Endpoint::Opaque,
            _ => Endpoint::Named(name),
        }
    }

    fn item_of(&self, bound: &TypeParamBound) -> Option<Endpoint> {
        match bound {
            TypeParamBound::Trait(found) => found.path.segments.last().and_then(|last| self.argument(&last.arguments)),
            _ => None,
        }
    }

    fn argument(&self, arguments: &PathArguments) -> Option<Endpoint> {
        match arguments {
            PathArguments::AngleBracketed(bracketed) => {
                bracketed.args.iter().find_map(|argument| self.generic(argument))
            }
            _ => None,
        }
    }

    fn generic(&self, argument: &GenericArgument) -> Option<Endpoint> {
        match argument {
            GenericArgument::Type(inner) => Some(self.endpoint(inner)),
            GenericArgument::AssocType(associated)
                if Name::from(&associated.ident).matches::<Whole>(&Self::ITEM).is_some() =>
            {
                Some(self.endpoint(&associated.ty))
            }
            _ => None,
        }
    }
}

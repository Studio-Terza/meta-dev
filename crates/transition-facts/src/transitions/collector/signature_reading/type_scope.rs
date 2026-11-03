mod bound;
mod constraint;

use std::collections::BTreeMap;

use syn::{GenericArgument, Generics, Path, PathArguments, PathSegment, Type, TypeParamBound};

use lint_layout::{Name, Whole, Word, words};

use super::super::origins::Origins;
use crate::endpoint::{Endpoint, Nature, Unnamed};
use bound::Bound;
use constraint::Constraint;

#[derive(Clone, Debug)]
pub(super) struct TypeScope<'file> {
    selfish: Endpoint,
    generics: BTreeMap<Name, Endpoint>,
    origins: &'file Origins,
}

impl<'file> TypeScope<'file> {
    const WRAPPERS: [Word<'static>; 14] = words![
        "Option", "Result", "Vec", "Box", "Rc", "Arc", "Cow", "VecDeque", "BTreeSet", "HashSet", "RefCell", "Cell",
        "Mutex", "Reverse"
    ];
    const PRIMITIVES: [Word<'static>; 18] = words![
        "bool", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64",
        "char", "str", "String"
    ];
    const SELF: [Word<'static>; 1] = words!["Self"];
    const SIZED: Word<'static> = Word::new("Sized");
    const FN: Word<'static> = Word::new("Fn");

    pub(super) fn new(selfish: Endpoint, origins: &'file Origins) -> TypeScope<'file> {
        TypeScope { selfish, generics: BTreeMap::new(), origins }
    }

    pub(super) fn extended(&self, generics: &Generics) -> TypeScope<'file> {
        let names: Vec<Name> = generics.type_params().map(|parameter| Name::from(&parameter.ident)).collect();
        let unbounded = names.iter().fold(self.clone(), |mut scope, name| {
            scope.generics.insert(name.clone(), Endpoint::new(Nature::Interface, Name::from(Self::SIZED)));
            scope
        });
        let constraints = Constraint::declared(generics);
        let bounded: Vec<Endpoint> =
            names.iter().map(|name| unbounded.bounded(Constraint::bounds(&constraints, name))).collect();
        let mut scope = unbounded;
        scope.generics.extend(names.into_iter().zip(bounded));
        scope
    }

    pub(super) fn endpoint(&self, found: &Type) -> Endpoint {
        match found {
            Type::Reference(reference) => self.endpoint(&reference.elem),
            Type::Paren(paren) => self.endpoint(&paren.elem),
            Type::Ptr(pointer) => self.endpoint(&pointer.elem),
            Type::Slice(slice) => self.endpoint(&slice.elem),
            Type::Array(array) => self.endpoint(&array.elem),
            Type::Tuple(tuple) if tuple.elems.is_empty() => Endpoint::from(Unnamed::Unit),
            Type::Tuple(tuple) => Endpoint::tupled(tuple.elems.iter().map(|element| self.endpoint(element))),
            Type::Path(path) => path
                .path
                .segments
                .last()
                .map_or_else(|| Endpoint::from(Unnamed::Opaque), |last| self.named(&path.path, last)),
            Type::ImplTrait(found) => self.bounded(found.bounds.iter()),
            Type::TraitObject(found) => self.bounded(found.bounds.iter()),
            Type::BareFn(_) => Endpoint::new(Nature::Interface, Name::from(Self::FN)),
            _ => Endpoint::from(Unnamed::Opaque),
        }
    }

    pub(super) fn argument(&self, arguments: &PathArguments) -> Option<Endpoint> {
        match arguments {
            PathArguments::AngleBracketed(bracketed) => {
                bracketed.args.iter().find_map(|argument| self.generic(argument))
            }
            _ => None,
        }
    }

    fn bounded<'bound>(&self, mut bounds: impl Iterator<Item = &'bound TypeParamBound>) -> Endpoint {
        bounds
            .find_map(|bound| Bound::new(bound).endpoint(self))
            .unwrap_or_else(|| Endpoint::new(Nature::Interface, Name::from(Self::SIZED)))
    }

    fn named(&self, path: &Path, segment: &PathSegment) -> Endpoint {
        let name = Name::from(&segment.ident);
        match name {
            _ if name.matches::<Whole>(&Self::WRAPPERS).is_some() => {
                self.argument(&segment.arguments).unwrap_or_else(|| Endpoint::from(Unnamed::Unit))
            }
            _ if name.matches::<Whole>(&Self::PRIMITIVES).is_some() => Endpoint::new(Nature::Primitive, name),
            _ if name.matches::<Whole>(&Self::SELF).is_some() => self.selfish.clone(),
            _ => self
                .generics
                .get(&name)
                .cloned()
                .unwrap_or_else(|| Endpoint::new(Nature::Named, name).originating(self.origins.of_path(path))),
        }
    }

    fn generic(&self, argument: &GenericArgument) -> Option<Endpoint> {
        match argument {
            GenericArgument::Type(inner) => Some(self.endpoint(inner)),
            GenericArgument::AssocType(associated) => Some(self.endpoint(&associated.ty)),
            _ => None,
        }
    }
}

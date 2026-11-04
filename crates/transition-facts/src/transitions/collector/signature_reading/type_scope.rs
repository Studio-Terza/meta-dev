mod bound;
mod constraint;
mod naming;

use std::collections::BTreeMap;

use syn::{FnArg, GenericArgument, Generics, PathArguments, Type, TypeParamBound, TypePath};

use lint_layout::{Name, Word};

use crate::associated::Associated;
use crate::endpoint::{Endpoint, Nature, Unnamed};
use crate::resolved::Resolved;
use crate::wrapping::{Bounds, Wrapping};
use bound::Bound;
use constraint::Constraint;
use naming::Naming;

#[derive(Debug)]
pub(super) struct TypeScope<'outer> {
    selfish: Nature,
    outer: Option<&'outer TypeScope<'outer>>,
    generics: BTreeMap<Name, Resolved>,
    wrappings: BTreeMap<Name, Wrapping>,
}

impl<'outer> TypeScope<'outer> {
    const SIZED: Word<'static> = Word::new("Sized");
    const FN: Word<'static> = Word::new("Fn");
    const ASSOCIATED: Word<'static> = Word::new("Self::Output");

    pub(super) fn root(selfish: Nature) -> TypeScope<'outer> {
        TypeScope { selfish, outer: None, generics: BTreeMap::new(), wrappings: BTreeMap::new() }
    }

    pub(super) fn declaring(
        selfish: Nature,
        outer: Option<&'outer TypeScope<'outer>>,
        generics: &Generics,
    ) -> TypeScope<'outer> {
        let unbounded = generics
            .type_params()
            .map(|parameter| (Name::from(&parameter.ident), Resolved::interface(Self::SIZED)))
            .collect();
        let constraints = Constraint::declared(generics);
        let wrappings = generics
            .type_params()
            .map(|parameter| Name::from(&parameter.ident))
            .map(|name| (name.clone(), Bounds::wrapping_of(Constraint::bounds(&constraints, &name))))
            .collect();
        let mut scope = TypeScope { selfish, outer, generics: unbounded, wrappings };
        let bounded: Vec<Resolved> =
            scope.generics.keys().map(|name| scope.bounded(Constraint::bounds(&constraints, name))).collect();
        scope.generics.values_mut().zip(bounded).for_each(|(slot, bound)| *slot = bound);
        scope
    }

    pub(super) fn associating(mut self, resolved: Resolved) -> TypeScope<'outer> {
        self.generics.insert(Name::from(Self::ASSOCIATED), resolved);
        self
    }

    pub(super) fn extended(&self, generics: &Generics) -> TypeScope<'_> {
        TypeScope::declaring(self.selfish, Some(self), generics)
    }

    pub(super) fn endpoint(&self, found: &Type) -> Resolved {
        match found {
            Type::Reference(reference) => self.endpoint(&reference.elem),
            Type::Paren(paren) => self.endpoint(&paren.elem),
            Type::Ptr(pointer) => self.endpoint(&pointer.elem),
            Type::Slice(slice) => self.endpoint(&slice.elem),
            Type::Array(array) => self.endpoint(&array.elem),
            Type::Tuple(tuple) => {
                Resolved::tupled(tuple.elems.iter().map(|element| self.endpoint(element)).collect(), self.selfish)
            }
            Type::Path(path) => self.named(path),
            Type::ImplTrait(found) => self.bounded(found.bounds.iter()),
            Type::TraitObject(found) => self.bounded(found.bounds.iter()),
            Type::BareFn(_) => Resolved::interface(Self::FN),
            _ => Resolved::Found(Endpoint::from(Unnamed::Opaque)),
        }
    }

    pub(super) fn taken(&self, input: &FnArg) -> Wrapping {
        match input {
            FnArg::Typed(typed) => Wrapping::peeled(&typed.ty, &|name| self.generic_wrapping(name)),
            FnArg::Receiver(_) => Wrapping::Plain,
        }
    }

    pub(super) fn generic_wrapping(&self, name: &Name) -> Option<Wrapping> {
        std::iter::successors(Some(self), |scope| scope.outer).find_map(|scope| scope.wrappings.get(name).copied())
    }

    pub(super) fn argument(&self, arguments: &PathArguments) -> Option<Resolved> {
        match arguments {
            PathArguments::AngleBracketed(bracketed) => {
                bracketed.args.iter().find_map(|argument| self.generic(argument))
            }
            _ => None,
        }
    }

    fn bounded<'bound>(&self, mut bounds: impl Iterator<Item = &'bound TypeParamBound>) -> Resolved {
        bounds.find_map(|bound| Bound::new(bound).endpoint(self)).unwrap_or_else(|| Resolved::interface(Self::SIZED))
    }

    fn named(&self, path: &TypePath) -> Resolved {
        Associated::of_path(path)
            .and_then(|_| self.declared(&Name::from(Self::ASSOCIATED)))
            .or_else(|| path.path.segments.last().map(|last| Naming::spelled(self, &path.path, last)))
            .unwrap_or_else(|| Resolved::Found(Endpoint::from(Unnamed::Opaque)))
    }

    fn declared(&self, name: &Name) -> Option<Resolved> {
        self.generics.get(name).cloned().or_else(|| self.outer.and_then(|outer| outer.declared(name)))
    }

    fn generic(&self, argument: &GenericArgument) -> Option<Resolved> {
        match argument {
            GenericArgument::Type(inner) => Some(self.endpoint(inner)),
            GenericArgument::AssocType(associated) => Some(self.endpoint(&associated.ty)),
            _ => None,
        }
    }
}

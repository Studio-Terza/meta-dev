use graph::{
    Bounds, Callable, Element, Grouped, ItemPath, Named, Outline, Projection, Resolved as Target, Sited, TypeRef,
};
use lint_layout::{Name, Whole, Word, words};

use super::Scope;
use super::endpoints::Endpoints;
use crate::transitions::wrapping::Wrapping;

#[derive(Clone, Copy, Debug)]
pub(super) struct Peeling<'scope, 'look, 'file> {
    scope: &'scope Scope<'look, 'file>,
}

impl<'scope, 'look, 'file> Peeling<'scope, 'look, 'file> {
    const TRANSPARENT: [Word<'static>; 8] = words!["Box", "Rc", "Arc", "Cow", "RefCell", "Cell", "Mutex", "Reverse"];
    const COLLECTIONS: [Word<'static>; 4] = words!["Vec", "VecDeque", "BTreeSet", "HashSet"];
    const OPTION: [Word<'static>; 1] = words!["Option"];
    const RESULT: [Word<'static>; 1] = words!["Result"];

    pub(super) fn new(scope: &'scope Scope<'look, 'file>) -> Peeling<'scope, 'look, 'file> {
        Peeling { scope }
    }

    fn inside(self, arguments: &[TypeRef]) -> Wrapping {
        arguments.first().map_or(Wrapping::Plain, |inner| inner.outlined(self))
    }

    fn wrapped(self, name: &Name, arguments: &[TypeRef]) -> Option<Wrapping> {
        let optional = name.matches::<Whole>(&Self::OPTION).map(|_| Wrapping::Optional.around(self.inside(arguments)));
        let fallible =
            || name.matches::<Whole>(&Self::RESULT).map(|_| Wrapping::Fallible.around(self.inside(arguments)));
        let many = || name.matches::<Whole>(&Self::COLLECTIONS).map(|_| Wrapping::Many);
        let transparent = || name.matches::<Whole>(&Self::TRANSPARENT).map(|_| self.inside(arguments));
        optional.or_else(fallible).or_else(many).or_else(transparent)
    }

    fn iterating(bound: &TypeRef) -> Option<Wrapping> {
        let target: &Target = bound.named()?.as_ref();
        target.std()?.last()?.matches::<Whole>(&Endpoints::ITERATORS).map(|_| Wrapping::Many)
    }
}

impl Outline for Peeling<'_, '_, '_> {
    type Kind = Wrapping;

    fn named(self, named: &Named<Sited>) -> Wrapping {
        let target: &Target = named.as_ref();
        let generic = target
            .parameter()
            .and_then(|name| self.scope.declared(name))
            .map(|declared| declared.bounds().iter().find_map(Self::iterating).unwrap_or(Wrapping::Plain));
        let wrapped = || target.std().and_then(ItemPath::last).and_then(|name| self.wrapped(name, named.arguments()));
        generic.or_else(wrapped).unwrap_or(Wrapping::Plain)
    }

    fn grouped(self, _parts: Grouped<'_>) -> Wrapping {
        Wrapping::Plain
    }

    fn sequence(self, _element: Element<'_>) -> Wrapping {
        Wrapping::Many
    }

    fn bounded(self, bounds: Bounds<'_>) -> Wrapping {
        bounds.as_ref().iter().find_map(Self::iterating).unwrap_or(Wrapping::Plain)
    }

    fn projected(self, projection: &Projection<Sited>) -> Wrapping {
        let output =
            projection.associated().and_then(|name| name.matches::<Whole>(&Endpoints::OUTPUT)).and(self.scope.output);
        output.map_or(Wrapping::Plain, |ty| ty.outlined(self))
    }

    fn callable(self, _callable: &Callable<Sited>) -> Wrapping {
        Wrapping::Plain
    }

    fn other(self) -> Wrapping {
        Wrapping::Plain
    }
}

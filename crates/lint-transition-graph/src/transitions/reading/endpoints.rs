use code_model::{
    Bounds, Callable, Element, Grouped, ItemPath, Named, Outline, Projection, Resolved as Target, Sited, TypeRef,
};
use lint_layout::{Name, Whole, Word, words};

use super::Scope;
use crate::transitions::endpoint::{Endpoint, Nature, Unnamed};
use crate::transitions::resolved::Resolved;

#[derive(Clone, Copy, Debug)]
pub(super) struct Endpoints<'scope, 'look, 'file> {
    scope: &'scope Scope<'look, 'file>,
}

impl<'scope, 'look, 'file> Endpoints<'scope, 'look, 'file> {
    const WRAPPERS: [Word<'static>; 14] = words![
        "Option", "Result", "Vec", "Box", "Rc", "Arc", "Cow", "VecDeque", "BTreeSet", "HashSet", "RefCell", "Cell",
        "Mutex", "Reverse"
    ];
    const STRINGS: [Word<'static>; 2] = words!["String", "str"];
    pub(super) const ITERATORS: [Word<'static>; 4] =
        words!["Iterator", "IntoIterator", "DoubleEndedIterator", "ExactSizeIterator"];
    const SELF: [Word<'static>; 1] = words!["Self"];
    pub(super) const OUTPUT: [Word<'static>; 1] = words!["Output"];
    const SIZED: Word<'static> = Word::new("Sized");
    const FN: Word<'static> = Word::new("Fn");
    const STD: Word<'static> = Word::new("std");

    pub(super) fn new(scope: &'scope Scope<'look, 'file>) -> Endpoints<'scope, 'look, 'file> {
        Endpoints { scope }
    }

    fn std(self, named: &Named<Sited>, path: &ItemPath) -> Resolved {
        let last = path.last().cloned().unwrap_or_default();
        let wrapped = last.matches::<Whole>(&Self::WRAPPERS).map(|_| {
            named
                .arguments()
                .first()
                .map_or(Resolved::Found(Endpoint::from(Unnamed::Unit)), |inner| inner.outlined(self))
        });
        let stringly = last
            .matches::<Whole>(&Self::STRINGS)
            .map(|_| Resolved::Found(Endpoint::new(Nature::Primitive, last.clone())));
        wrapped.or(stringly).unwrap_or_else(|| {
            Resolved::Found(Endpoint::new(Nature::Named, last.clone()).originating(Some(Name::from(Self::STD))))
        })
    }

    fn parameter(self, name: &Name) -> Resolved {
        let own = name.matches::<Whole>(&Self::SELF).map(|_| Resolved::Itself);
        own.unwrap_or_else(|| {
            self.scope.declared(name).map_or_else(
                || Resolved::interface(Self::SIZED),
                |parameter| self.bounded(Bounds::from(parameter.bounds())),
            )
        })
    }

    fn written(named: &Named<Sited>, origin: Option<Name>) -> Resolved {
        let name = named.last_spelled().cloned().unwrap_or_default();
        Resolved::Found(Endpoint::new(Nature::Named, name).originating(origin))
    }

    fn bound(self, bound: &TypeRef) -> Option<Resolved> {
        let named = bound.named()?;
        let target: &Target = named.as_ref();
        let last = target.std().and_then(ItemPath::last).or_else(|| named.last_spelled())?;
        let iterating = last.matches::<Whole>(&Self::ITERATORS).map(|_| {
            named
                .iterated()
                .map_or(Resolved::Found(Endpoint::from(Unnamed::Opaque)), |inner| inner.as_ref().outlined(self))
        });
        Some(iterating.unwrap_or_else(|| Resolved::Found(Endpoint::new(Nature::Interface, last.clone()))))
    }
}

impl Outline for Endpoints<'_, '_, '_> {
    type Kind = Resolved;

    fn named(self, named: &Named<Sited>) -> Resolved {
        let target: &Target = named.as_ref();
        let parameter = target.parameter().map(|name| self.parameter(name));
        let std = || target.std().map(|path| self.std(named, path));
        let primitive = || {
            target.primitive().map(|_| {
                Resolved::Found(Endpoint::new(Nature::Primitive, named.last_spelled().cloned().unwrap_or_default()))
            })
        };
        let declared = || {
            target.crate_item().and_then(|id| self.scope.reading.declared(id)).map(|found| {
                Resolved::Found(Endpoint::new(Nature::Named, found.name.clone()).originating(found.krate.cloned()))
            })
        };
        let external = || {
            target.external().map(|found| Self::written(named, AsRef::<[Name]>::as_ref(found.path()).first().cloned()))
        };
        parameter
            .or_else(std)
            .or_else(primitive)
            .or_else(declared)
            .or_else(external)
            .unwrap_or_else(|| Self::written(named, None))
    }

    fn grouped(self, parts: Grouped<'_>) -> Resolved {
        let selfish = self.scope.selfish;
        let elements = parts.as_ref();
        elements.first().map_or(Resolved::Found(Endpoint::from(Unnamed::Unit)), |_| {
            elements
                .iter()
                .map(|part| part.outlined(self))
                .find(|resolved| resolved.nature(selfish) == Nature::Primitive)
                .unwrap_or(Resolved::Found(Endpoint::from(Unnamed::Opaque)))
        })
    }

    fn sequence(self, element: Element<'_>) -> Resolved {
        element.as_ref().outlined(self)
    }

    fn bounded(self, bounds: Bounds<'_>) -> Resolved {
        bounds.as_ref().iter().find_map(|bound| self.bound(bound)).unwrap_or_else(|| Resolved::interface(Self::SIZED))
    }

    fn projected(self, projection: &Projection<Sited>) -> Resolved {
        let associated = projection.associated();
        let output = associated
            .and_then(|name| name.matches::<Whole>(&Self::OUTPUT))
            .and(self.scope.output)
            .map(|ty| ty.outlined(self));
        output.unwrap_or(Resolved::Found(Endpoint::from(Unnamed::Opaque)))
    }

    fn callable(self, _callable: &Callable<Sited>) -> Resolved {
        Resolved::interface(Self::FN)
    }

    fn other(self) -> Resolved {
        Resolved::Found(Endpoint::from(Unnamed::Opaque))
    }
}

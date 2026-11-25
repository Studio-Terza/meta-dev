use graph::{
    Bounds, Callable, Element, Grouped, ItemKind, Mutated, Named, Outline, Projection, Resolved as Target, Signature,
    Sited, TypeRef,
};
use lint_layout::{Whole, Word, words};

use super::Reading;
use crate::transitions::handle::Handle;

#[derive(Clone, Copy, Debug)]
pub(super) struct Porting<'look, 'file> {
    reading: &'look Reading<'file>,
}

impl<'look, 'file> From<&'look Reading<'file>> for Porting<'look, 'file> {
    fn from(reading: &'look Reading<'file>) -> Porting<'look, 'file> {
        Porting { reading }
    }
}

impl Porting<'_, '_> {
    const CRATE: [Word<'static>; 1] = words!["store_ports"];
    const TRAIT: [Word<'static>; 1] = words!["Execute"];

    pub(super) fn held(self, signature: &Signature) -> Handle {
        let mutated: &[TypeRef] = AsRef::<Mutated>::as_ref(signature).as_ref();
        let ports: Option<Vec<()>> = mutated.iter().map(|ty| self.reading.model.expanded(ty).outlined(self)).collect();
        ports.and_then(|found| found.into_iter().next()).map_or(Handle::Owned, |()| Handle::Port)
    }

    fn store_port(self, bound: &TypeRef) -> Option<()> {
        let target: &Target = bound.named()?.as_ref();
        let id = target.crate_item()?;
        self.reading.model.declared(id).filter(|item| item.kind() == ItemKind::Trait)?;
        let homed = self.reading.declared(id)?;
        homed.name.matches::<Whole>(&Self::TRAIT)?;
        homed.krate?.matches::<Whole>(&Self::CRATE).map(|_| ())
    }
}

impl Outline for Porting<'_, '_> {
    type Kind = Option<()>;

    fn named(self, _named: &Named<Sited>) -> Option<()> {
        None
    }

    fn grouped(self, _parts: Grouped<'_>) -> Option<()> {
        None
    }

    fn sequence(self, _element: Element<'_>) -> Option<()> {
        None
    }

    fn bounded(self, bounds: Bounds<'_>) -> Option<()> {
        bounds.as_ref().iter().find_map(|bound| self.store_port(bound))
    }

    fn projected(self, _projection: &Projection<Sited>) -> Option<()> {
        None
    }

    fn callable(self, _callable: &Callable<Sited>) -> Option<()> {
        None
    }

    fn other(self) -> Option<()> {
        None
    }
}

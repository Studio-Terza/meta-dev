use std::borrow::Borrow;
use std::collections::BTreeMap;

use body_shape::{Bounds, Erasure, Labels};
use equivalence::Equivalence;
use lint_core::{Parsed, Syntax, Walk};
use lint_layout::Name;
use lint_placement::CrateName;
use lint_sweep::{Descend, Facts, Gathers, Kind, Sweep, Watch, Watching};
use syntax::{ItemImpl, ItemMod};

use crate::end::Node;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Filled<'file> {
    krate: &'file CrateName,
    filler: Name,
    interface: Name,
}

#[derive(Debug, Default)]
pub(crate) struct Fillers<'file> {
    bodies: BTreeMap<Filled<'file>, BTreeMap<Name, Labels>>,
}

impl<'file> Fillers<'file> {
    pub(crate) fn read(parsed: Parsed<'_, 'file>, equivalence: Equivalence) -> Fillers<'file> {
        let shipped = parsed.file().shipped().map(|_| parsed);
        shipped.map_or_default(|found| {
            let krate = found.head().placement().package().crate_name();
            let erasure = Erasure::reading(found.file().tokens(), equivalence);
            Sweep::gathered(found.file(), Scan { krate, erasure, fillers: Fillers::default() })
        })
    }

    pub(crate) fn alike<'found>(&self, first: Node<'found>, other: Node<'found>) -> Option<Node<'found>> {
        let ours = self.filled(&first);
        let theirs = self.filled(&other);
        ours.iter()
            .find_map(|(interface, methods)| theirs.get(interface).filter(|their| **their != *methods))
            .map_or(Some(other), |_| None)
    }

    fn filled(&self, node: &Node<'_>) -> BTreeMap<&Name, &BTreeMap<Name, Labels>> {
        self.bodies
            .iter()
            .filter(|(filled, _)| Borrow::<Name>::borrow(filled.krate) == node.krate())
            .filter(|(filled, _)| filled.filler == *AsRef::<Name>::as_ref(node))
            .map(|(filled, methods)| (&filled.interface, methods))
            .collect()
    }
}

#[derive(Debug)]
struct Scan<'syntax, 'file> {
    krate: &'file CrateName,
    erasure: Erasure<'syntax>,
    fillers: Fillers<'file>,
}

impl<'file> Gathers<Fillers<'file>> for Scan<'_, 'file> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> Fillers<'file> {
        self.fillers
    }
}

impl<'ast> Watch<'ast> for Scan<'_, '_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemImpl]
    }

    fn on_item_mod(&mut self, _: &'ast ItemMod, facts: &Facts<'_, ItemMod>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_item_impl(&mut self, found: &'ast ItemImpl, facts: &Facts<'_, ItemImpl>) -> Descend {
        let interface =
            found.trait_.as_ref().and_then(|(_, path, _)| path.segments.last()).map(|last| Name::from(&last.ident));
        let filled = interface.zip(facts.name().cloned()).map(|(trait_name, filler)| Filled {
            krate: self.krate,
            filler,
            interface: trait_name,
        });
        let methods = found
            .items
            .iter()
            .filter_map(Syntax::impl_fn)
            .map(|method| {
                (Name::from(&method.sig.ident), self.erasure.read(Bounds::from(&method.block)).labels().clone())
            })
            .collect();
        self.fillers.bodies.extend(filled.filter(|_| facts.gate() == Walk::Entered).map(|kept| (kept, methods)));
        Descend::Over
    }
}

impl<'file> FromIterator<Fillers<'file>> for Fillers<'file> {
    fn from_iter<Found: IntoIterator<Item = Fillers<'file>>>(parts: Found) -> Fillers<'file> {
        Fillers { bodies: parts.into_iter().flat_map(|part| part.bodies).collect() }
    }
}

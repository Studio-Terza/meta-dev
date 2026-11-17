use lint_core::Extended;
use lint_core::SourceFile;
use std::borrow::Borrow;
use std::collections::BTreeMap;

use body_shape::{Bounds, Erasure, Labels};
use equivalence::Equivalence;
use lint_core::{Parsed, Syntax, Walk};
use lint_layout::Name;
use lint_placement::CrateName;
use lint_sweep::{Descend, Facts, Fold, Folding, Kind, Sweep, Visit};
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
        let shipped = <&SourceFile>::from(parsed).shipped().map(|_| parsed);
        shipped.map_or_default(|found| {
            let krate = found.head().placement().package().crate_name();
            let erasure = Erasure::reading(<&SourceFile>::from(found).tokens(), equivalence);
            Sweep::gathered(<&SourceFile>::from(found), Folding::from(Scan { krate, erasure }))
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
}

impl<'file> Fold for Scan<'_, 'file> {
    type Found = Fillers<'file>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemImpl]
    }

    fn on_item_mod(&self, fillers: Self::Found, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Self::Found> {
        Visit::new(fillers, Descend::from(facts.gate()))
    }

    fn on_item_impl(&self, fillers: Self::Found, found: &ItemImpl, facts: &Facts<'_, ItemImpl>) -> Visit<Self::Found> {
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
        let kept = filled.filter(|_| facts.gate() == Walk::Entered).map(|kept| (kept, methods));
        Visit::new(Fillers { bodies: fillers.bodies.extended(kept) }, Descend::Over)
    }
}

impl<'file> FromIterator<Fillers<'file>> for Fillers<'file> {
    fn from_iter<Found: IntoIterator<Item = Fillers<'file>>>(parts: Found) -> Fillers<'file> {
        Fillers { bodies: parts.into_iter().flat_map(|part| part.bodies).collect() }
    }
}

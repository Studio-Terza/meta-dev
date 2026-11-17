use lint_core::SourceFile;
use std::borrow::Borrow;
use std::collections::BTreeMap;

use algebra::{Monoid, Semigroup};
use body_shape::{Bounds, Erasure, Labels};
use equivalence::Equivalence;
use lint_core::{Parsed, Syntax, Walk};
use lint_layout::Name;
use lint_placement::CrateName;
use lint_sweep::{Contribute, Contributed, Contributing, Descend, Facts, Gathers, Kind, Sweep, Watching};
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
            Sweep::gathered(<&SourceFile>::from(found), Contributing::new(Scan { krate, erasure }))
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

impl<'file> Gathers<Fillers<'file>> for Contributing<Scan<'_, 'file>> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> Fillers<'file> {
        self.concluded(|_, (), fillers| fillers)
    }
}

impl<'file> Contribute for Scan<'_, 'file> {
    type Open = ();
    type Part = Fillers<'file>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemImpl]
    }

    fn on_item_mod(&self, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Contributed<(), Fillers<'file>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_impl(&self, found: &ItemImpl, facts: &Facts<'_, ItemImpl>) -> Contributed<(), Fillers<'file>> {
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
        let bodies = filled.filter(|_| facts.gate() == Walk::Entered).map(|kept| (kept, methods)).into_iter().collect();
        Contributed::new((), Fillers { bodies }, Descend::Over)
    }
}

impl<'file> FromIterator<Fillers<'file>> for Fillers<'file> {
    fn from_iter<Found: IntoIterator<Item = Fillers<'file>>>(parts: Found) -> Fillers<'file> {
        Monoid::concat(parts)
    }
}

impl Semigroup for Fillers<'_> {
    fn combine(mut self, other: Self) -> Self {
        self.bodies.extend(other.bodies);
        self
    }
}

impl Monoid for Fillers<'_> {}

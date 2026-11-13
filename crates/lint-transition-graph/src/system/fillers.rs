use std::borrow::Borrow;
use std::collections::BTreeMap;

use body_shape::{Reading, Shape};
use equivalence::Equivalence;
use lint_core::{Parsed, Syntax, Walk};
use lint_layout::Name;
use lint_placement::CrateName;
use lint_sweep::{Descend, Gathers, Kind, Sweep, Watch, Watching};
use quote::ToTokens;
use syn::{ItemImpl, ItemMod};

use crate::end::Node;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Filled<'file> {
    krate: &'file CrateName,
    filler: Name,
    interface: Name,
}

#[derive(Debug, Default)]
pub(crate) struct Fillers<'file> {
    bodies: BTreeMap<Filled<'file>, BTreeMap<Name, Shape>>,
}

impl<'file> Fillers<'file> {
    pub(crate) fn read(parsed: Parsed<'_, 'file>, equivalence: Equivalence) -> Fillers<'file> {
        let shipped = parsed.file().shipped().map(|_| parsed);
        shipped.map_or_default(|found| {
            let krate = found.head().placement().package().crate_name();
            Sweep::gathered(found.file(), Scan { krate, equivalence, fillers: Fillers::default() })
        })
    }

    pub(crate) fn alike<'found>(&self, first: Node<'found>, other: Node<'found>) -> Option<Node<'found>> {
        let ours = self.filled(&first);
        let theirs = self.filled(&other);
        ours.iter()
            .find_map(|(interface, methods)| theirs.get(interface).filter(|their| **their != *methods))
            .map_or(Some(other), |_| None)
    }

    fn filled(&self, node: &Node<'_>) -> BTreeMap<&Name, &BTreeMap<Name, Shape>> {
        self.bodies
            .iter()
            .filter(|(filled, _)| Borrow::<Name>::borrow(filled.krate) == node.krate())
            .filter(|(filled, _)| filled.filler == *AsRef::<Name>::as_ref(node))
            .map(|(filled, methods)| (&filled.interface, methods))
            .collect()
    }
}

#[derive(Debug)]
struct Scan<'file> {
    krate: &'file CrateName,
    equivalence: Equivalence,
    fillers: Fillers<'file>,
}

impl<'file> Gathers<Fillers<'file>> for Scan<'file> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> Fillers<'file> {
        self.fillers
    }
}

impl<'ast> Watch<'ast> for Scan<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemMod, Kind::ItemImpl]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_impl(&mut self, found: &'ast ItemImpl) -> Descend {
        let interface =
            found.trait_.as_ref().and_then(|(_, path, _)| path.segments.last()).map(|last| Name::from(&last.ident));
        let filled = interface.zip(Name::of_type(&found.self_ty)).map(|(trait_name, filler)| Filled {
            krate: self.krate,
            filler,
            interface: trait_name,
        });
        let methods = found
            .items
            .iter()
            .filter_map(Syntax::impl_fn)
            .map(|method| {
                (
                    Name::from(&method.sig.ident),
                    Reading::erasing(method.block.to_token_stream(), self.equivalence).shape(),
                )
            })
            .collect();
        self.fillers
            .bodies
            .extend(filled.filter(|_| Walk::outside_tests(&found.attrs) == Walk::Entered).map(|kept| (kept, methods)));
        Descend::Over
    }
}

impl<'file> FromIterator<Fillers<'file>> for Fillers<'file> {
    fn from_iter<Found: IntoIterator<Item = Fillers<'file>>>(parts: Found) -> Fillers<'file> {
        Fillers { bodies: parts.into_iter().flat_map(|part| part.bodies).collect() }
    }
}

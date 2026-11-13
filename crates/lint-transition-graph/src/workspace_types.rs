use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};

use crate::transitions::endpoint::Nature;
use crate::transitions::{Endpoint, Implementer, Inputs, Output, Transition, Transitions};
use code_model::{ItemKind, Model, Reach, Region};
use lint_layout::Name;
use lint_placement::CrateName;

use crate::count::Count;
use crate::edge::Edge;
use crate::end::{End, Ends, Node, Place};

#[derive(Debug)]
struct Spotted<'found> {
    krate: &'found Name,
    name: &'found Name,
    kind: ItemKind,
}

#[derive(Debug, Default)]
pub(crate) struct WorkspaceTypes<'found> {
    types: BTreeMap<&'found Name, BTreeSet<&'found Name>>,
    traits: BTreeSet<&'found Name>,
    crates: BTreeSet<&'found Name>,
    ambiguous: BTreeMap<&'found Name, BTreeSet<&'found Name>>,
}

impl<'found> WorkspaceTypes<'found> {
    pub(crate) const TYPES: [ItemKind; 2] = [ItemKind::Struct, ItemKind::Enum];

    pub(crate) fn declared(model: &'found Model, transitions: &'found Transitions<'_>) -> WorkspaceTypes<'found> {
        let declared: Vec<Spotted<'found>> = model
            .items()
            .filter(|placed| Some(AsRef::<Region>::as_ref(placed.entity()).reach()) == Some(Reach::Shipped))
            .filter_map(|placed| {
                let item = model.declared(placed.entity().declared())?;
                Some(Spotted { krate: placed.krate(), name: item.as_ref(), kind: item.kind() })
            })
            .collect();
        let types: Vec<&Spotted<'found>> = declared.iter().filter(|found| Self::TYPES.contains(&found.kind)).collect();
        WorkspaceTypes {
            crates: transitions
                .all()
                .map(|found| Borrow::<Name>::borrow(AsRef::<CrateName>::as_ref(found.context())))
                .chain(types.iter().map(|found| found.krate))
                .collect(),
            ambiguous: Self::ambiguous(&types),
            types: types.iter().fold(BTreeMap::new(), |mut names: BTreeMap<&Name, BTreeSet<&Name>>, found| {
                names.entry(found.name).or_default().insert(found.krate);
                names
            }),
            traits: declared.iter().filter(|found| found.kind == ItemKind::Trait).map(|found| found.name).collect(),
        }
    }

    fn ambiguous(types: &[&Spotted<'found>]) -> BTreeMap<&'found Name, BTreeSet<&'found Name>> {
        let mut seen: BTreeMap<&Name, BTreeSet<&Name>> = BTreeMap::new();
        types.iter().fold(BTreeMap::new(), |mut twice: BTreeMap<&'found Name, BTreeSet<&'found Name>>, found| {
            let again = seen.entry(found.krate).or_default().replace(found.name);
            twice.entry(found.krate).or_default().extend(again);
            twice
        })
    }

    pub(crate) fn declared_in(&self, krate: &CrateName) -> Count {
        let name: &Name = krate.borrow();
        Count::new(self.types.values().filter(|crates| crates.contains(name)).count())
    }

    pub(crate) fn edge<'file>(&self, transition: Transition<'found, 'file>) -> Edge<'found, 'file> {
        let fixed_by =
            transition.context().trait_name().and_then(|name| self.traits.get(name).map_or(Some(name), |_| None));
        let from: &Name = AsRef::<CrateName>::as_ref(&transition).borrow();
        let end = |endpoint: &'found Endpoint| self.end(from, endpoint, fixed_by);
        let implementer = transition.implementer().map(Implementer::endpoint).map(end);
        let ends =
            Ends::new(Inputs::iter(transition.takes()).map(end).collect(), end(Output::endpoint(transition.gives())))
                .implemented_by(implementer);
        Edge::new(transition, ends, fixed_by)
    }

    pub(crate) fn owner(&self, transition: &Transition<'found, '_>) -> Option<End<'found>> {
        let context = transition.context();
        let from: &Name = AsRef::<CrateName>::as_ref(transition).borrow();
        context.trait_name().map_or(context.subject(), |_| None).map(|subject| self.end(from, subject, None))
    }

    fn end(&self, from: &Name, endpoint: &'found Endpoint, fixed_by: Option<&'found Name>) -> End<'found> {
        match endpoint.nature() {
            Nature::Named => self.named(endpoint.origin().map_or(from, Borrow::borrow), endpoint.as_ref()),
            Nature::Interface => End::placed(Place::Interface, endpoint.as_ref()),
            Nature::Primitive => fixed_by.map_or_else(
                || End::placed(Place::Primitive, endpoint.as_ref()),
                |fixing| End::placed(Place::Interface, fixing),
            ),
            Nature::Boxed => End::placed(Place::External, endpoint.as_ref()),
        }
    }

    fn named(&self, from: &Name, name: &'found Name) -> End<'found> {
        let declaring = self.types.get(name).filter(|_| self.crates.contains(from));
        let resolved = declaring.map(|crates| Node::resolved(crates, from, name));
        resolved.map_or(End::placed(Place::External, name), |found| {
            found.and_then(|node| self.unambiguous(node)).map_or(End::placed(Place::Shared, name), End::domain)
        })
    }

    fn unambiguous(&self, node: Node<'found>) -> Option<Node<'found>> {
        let twice = self.ambiguous.get(node.krate());
        twice.and_then(|names| names.get(AsRef::<Name>::as_ref(&node))).map_or(Some(node), |_| None)
    }
}

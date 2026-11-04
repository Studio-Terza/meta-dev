use std::collections::BTreeSet;
use std::fmt;

use crate::class::Class;
use crate::count::Count;
use crate::key::Key;
use lint_layout::{Name, Whole, Word};
use lint_placement::CrateName;
use transition_facts::{Mutation, Wrapping};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node<'found> {
    krate: &'found CrateName,
    name: &'found Name,
}

impl<'found> Node<'found> {
    pub(crate) fn resolved(
        declaring: &BTreeSet<&'found CrateName>,
        from: &CrateName,
        name: &'found Name,
    ) -> Option<Node<'found>> {
        let krate = declaring.get(from).or_else(|| declaring.first().filter(|_| declaring.len() == 1));
        krate.map(|krate| Node { krate, name })
    }
}

impl AsRef<CrateName> for Node<'_> {
    fn as_ref(&self) -> &CrateName {
        self.krate
    }
}

impl AsRef<Name> for Node<'_> {
    fn as_ref(&self) -> &Name {
        self.name
    }
}

impl fmt::Display for Node<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}", self.krate, self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Tuple<'found>(BTreeSet<Node<'found>>);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Hop<'found> {
    takes: Tuple<'found>,
    gives: Node<'found>,
}

impl<'found> Hop<'found> {
    pub(crate) fn joins(&self, first: &Hop<'found>, last: &Hop<'found>) -> Option<&Hop<'found>> {
        Some(self).filter(|hop| hop.takes == first.takes).filter(|hop| hop.gives == last.gives)
    }

    pub(crate) fn distinct<'ends>(ends: impl Iterator<Item = &'ends Ends<'found>>) -> Count
    where
        'found: 'ends,
    {
        Count::new(ends.filter_map(Option::<Hop>::from).collect::<BTreeSet<Hop<'found>>>().len())
    }
}

impl<'found> From<&Ends<'found>> for Option<Hop<'found>> {
    fn from(ends: &Ends<'found>) -> Option<Hop<'found>> {
        let takes: BTreeSet<Node<'found>> = ends.taken().copied().collect();
        let gives = ends.gives.node?;
        let first = takes.first().copied();
        first.map(|_| Hop { takes: Tuple(takes), gives })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Place {
    Domain,
    Shared,
    Interface,
    External,
    Primitive,
}

impl Place {
    fn written(self, name: &Name, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Place::Interface => write!(formatter, "impl {name}"),
            Place::Domain | Place::Shared | Place::External | Place::Primitive => write!(formatter, "{name}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct End<'found> {
    place: Place,
    name: &'found Name,
    node: Option<Node<'found>>,
}

impl<'found> End<'found> {
    const UNIT: Word<'static> = Word::new("()");

    pub(crate) fn placed(place: Place, name: &'found Name) -> End<'found> {
        End { place, name, node: None }
    }

    pub(crate) fn domain(node: Node<'found>) -> End<'found> {
        End { place: Place::Domain, name: node.name, node: Some(node) }
    }

    #[must_use]
    pub fn node(&self) -> Option<&Node<'found>> {
        self.node.as_ref()
    }

    pub(crate) fn unit(&self) -> Option<&End<'found>> {
        Some(self).filter(|end| end.name.matches::<Whole>(&[Self::UNIT]).is_some())
    }
}

impl AsRef<Name> for End<'_> {
    fn as_ref(&self) -> &Name {
        self.name
    }
}

impl fmt::Display for End<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.node {
            Some(node) => write!(formatter, "{node}"),
            None => self.place.written(self.name, formatter),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Ends<'found> {
    takes: Vec<End<'found>>,
    gives: End<'found>,
    implementer: Option<End<'found>>,
}

impl<'found> Ends<'found> {
    pub(crate) fn new(takes: Vec<End<'found>>, gives: End<'found>) -> Ends<'found> {
        Ends { takes, gives, implementer: None }
    }

    pub(crate) fn implemented_by(self, implementer: Option<End<'found>>) -> Ends<'found> {
        Ends { implementer, ..self }
    }

    pub(crate) fn placed(&self, place: Place) -> impl Iterator<Item = &End<'found>> {
        self.takes.iter().chain([&self.gives]).filter(move |end| end.place == place)
    }

    pub(crate) fn takes(&self) -> &[End<'found>] {
        &self.takes
    }

    pub(crate) fn subject(&self, mutation: Mutation, wrapping: Wrapping) -> Option<Node<'found>> {
        let receiver = Some(mutation)
            .filter(|mutation| *mutation == Mutation::ThroughSelf)
            .filter(|_| wrapping == Wrapping::Plain)
            .and_then(|_| self.gives.unit())
            .and_then(|_| self.takes.first().or(self.implementer.as_ref()));
        self.gives.node.or_else(|| receiver.and_then(|end| end.node))
    }

    pub(crate) fn taken(&self) -> impl Iterator<Item = &Node<'found>> {
        self.takes.iter().filter_map(|taken| taken.node.as_ref())
    }

    pub(crate) fn class(&self) -> Class {
        match self.placed(Place::Primitive).next() {
            Some(_) => Class::Primitive,
            None => Class::Covered,
        }
    }

    pub(crate) fn key(&self) -> Option<Key<'found>> {
        let mut takes: Vec<Node<'found>> = self.takes.iter().map(|taken| taken.node).collect::<Option<Vec<Node>>>()?;
        let gives = self.gives.node?;
        let selector = self
            .implementer
            .as_ref()
            .and_then(|implementer| implementer.node)
            .filter(|implementer| *implementer != gives)
            .filter(|implementer| takes.iter().all(|taken| taken != implementer));
        takes.extend(selector);
        takes.sort();
        let first = takes.first().copied();
        first.map(|_| Key::new(takes, gives))
    }

    pub(crate) fn applied(&self) -> Applied<'_, 'found> {
        Applied(self)
    }
}

impl<'found> AsRef<End<'found>> for Ends<'found> {
    fn as_ref(&self) -> &End<'found> {
        &self.gives
    }
}

#[derive(Debug)]
pub(crate) struct Applied<'ends, 'found>(&'ends Ends<'found>);

impl fmt::Display for Applied<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let takes: Vec<String> = self.0.takes.iter().skip(1).map(ToString::to_string).collect();
        write!(formatter, "{} -> {}", takes.join(", "), self.0.gives)
    }
}

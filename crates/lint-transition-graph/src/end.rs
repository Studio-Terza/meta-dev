use std::collections::BTreeSet;
use std::fmt;

use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::Ground;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Node<'found> {
    krate: &'found CrateName,
    name: &'found Name,
}

impl<'found> Node<'found> {
    pub(crate) fn declared(krate: &'found CrateName, name: &'found Name) -> Node<'found> {
        Node { krate, name }
    }

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

impl<'found> Tuple<'found> {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Node<'found>> {
        self.0.iter()
    }

    pub(crate) fn within<'tuple>(&'tuple self, available: &BTreeSet<&Node<'_>>) -> Option<&'tuple Tuple<'found>> {
        self.0.iter().all(|node| available.contains(node)).then_some(self)
    }
}

impl fmt::Display for Tuple<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nodes: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        match nodes.as_slice() {
            [single] => write!(formatter, "{single}"),
            _ => write!(formatter, "({})", nodes.join(", ")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Hop<'found> {
    takes: Tuple<'found>,
    gives: Node<'found>,
}

impl<'found> Hop<'found> {
    pub(crate) fn reached(&self) -> &Node<'found> {
        &self.gives
    }

    pub(crate) fn onward(self) -> Option<Hop<'found>> {
        (!self.takes.0.contains(&self.gives)).then_some(self)
    }

    pub(crate) fn fired(&self, available: &BTreeSet<&Node<'_>>) -> Option<&Node<'found>> {
        self.takes.within(available).map(|_| &self.gives)
    }
}

impl<'found> AsRef<Tuple<'found>> for Hop<'found> {
    fn as_ref(&self) -> &Tuple<'found> {
        &self.takes
    }
}

impl fmt::Display for Hop<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} -> {}", self.takes, self.gives)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Key<'found> {
    takes: Vec<Node<'found>>,
    gives: Node<'found>,
}

impl<'found> Key<'found> {
    pub(crate) fn algebraic(&self) -> Option<&Key<'found>> {
        self.takes.iter().all(|taken| *taken == self.gives).then_some(self)
    }
}

impl fmt::Display for Key<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let takes: Vec<String> = self.takes.iter().map(ToString::to_string).collect();
        write!(formatter, "{} -> {}", takes.join(", "), self.gives)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct End<'found> {
    place: Place,
    name: &'found Name,
    node: Option<Node<'found>>,
}

impl<'found> End<'found> {
    pub(crate) fn placed(place: Place, name: &'found Name) -> End<'found> {
        End { place, name, node: None }
    }

    pub(crate) fn domain(node: Node<'found>) -> End<'found> {
        End { place: Place::Domain, name: node.name, node: Some(node) }
    }

    pub(crate) fn place(&self) -> Place {
        self.place
    }

    pub(crate) fn node(&self) -> Option<&Node<'found>> {
        self.node.as_ref()
    }
}

impl fmt::Display for End<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.node, self.place) {
            (Some(node), _) => write!(formatter, "{node}"),
            (None, Place::Interface) => write!(formatter, "impl {}", self.name),
            (None, _) => write!(formatter, "{}", self.name),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Class {
    Covered,
    Primitive,
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

    pub(crate) fn all(&self) -> impl Iterator<Item = &End<'found>> {
        self.takes.iter().chain([&self.gives])
    }

    pub(crate) fn hop(&self) -> Option<Hop<'found>> {
        let takes: BTreeSet<Node<'found>> = self.taken().copied().collect();
        let gives = self.gives.node?;
        (!takes.is_empty()).then_some(Hop { takes: Tuple(takes), gives })
    }

    pub(crate) fn taken(&self) -> impl Iterator<Item = &Node<'found>> {
        self.takes.iter().filter_map(|taken| taken.node.as_ref())
    }

    pub(crate) fn entered(&self, ground: Ground) -> Option<&Node<'found>> {
        match (ground, self.taken().next()) {
            (Ground::Ports | Ground::Boundary, _) | (_, None) => self.gives.node.as_ref(),
            (Ground::Core | Ground::Store, Some(_)) => None,
        }
    }

    pub(crate) fn class(&self) -> Class {
        match self.all().find(|end| end.place == Place::Primitive) {
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
            .filter(|implementer| *implementer != gives && !takes.contains(implementer));
        takes.extend(selector);
        takes.sort();
        (!takes.is_empty()).then_some(Key { takes, gives })
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

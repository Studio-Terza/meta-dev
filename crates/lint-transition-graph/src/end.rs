use std::collections::BTreeSet;
use std::fmt;

use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::Ground;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Node {
    krate: CrateName,
    name: Name,
}

impl Node {
    pub(crate) fn declared(krate: &CrateName, name: &Name) -> Node {
        Node { krate: krate.clone(), name: name.clone() }
    }

    pub(crate) fn resolved(declaring: &BTreeSet<CrateName>, from: &CrateName, name: &Name) -> Option<Node> {
        let krate =
            if declaring.contains(from) { Some(from) } else { declaring.first().filter(|_| declaring.len() == 1) };
        krate.map(|krate| Node { krate: krate.clone(), name: name.clone() })
    }
}

impl fmt::Display for Node {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}", self.krate, self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Tuple(BTreeSet<Node>);

impl Tuple {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Node> {
        self.0.iter()
    }

    pub(crate) fn first(&self) -> Option<&Node> {
        self.0.first()
    }

    pub(crate) fn within<'tuple>(&'tuple self, available: &BTreeSet<&Node>) -> Option<&'tuple Tuple> {
        self.0.iter().all(|node| available.contains(node)).then_some(self)
    }
}

impl fmt::Display for Tuple {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nodes: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        match nodes.as_slice() {
            [single] => write!(formatter, "{single}"),
            _ => write!(formatter, "({})", nodes.join(", ")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Hop {
    takes: Tuple,
    gives: Node,
}

impl Hop {
    pub(crate) fn reached(&self) -> &Node {
        &self.gives
    }

    pub(crate) fn onward(self) -> Option<Hop> {
        (!self.takes.0.contains(&self.gives)).then_some(self)
    }

    pub(crate) fn fired(&self, available: &BTreeSet<&Node>) -> Option<&Node> {
        self.takes.within(available).map(|_| &self.gives)
    }
}

impl AsRef<Tuple> for Hop {
    fn as_ref(&self) -> &Tuple {
        &self.takes
    }
}

impl fmt::Display for Hop {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} -> {}", self.takes, self.gives)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Key {
    takes: Vec<Node>,
    gives: Node,
}

impl fmt::Display for Key {
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct End {
    place: Place,
    name: Name,
    node: Option<Node>,
}

impl End {
    pub(crate) fn placed(place: Place, name: &Name) -> End {
        End { place, name: name.clone(), node: None }
    }

    pub(crate) fn domain(node: Node) -> End {
        End { place: Place::Domain, name: node.name.clone(), node: Some(node) }
    }

    pub(crate) fn place(&self) -> Place {
        self.place
    }

    pub(crate) fn node(&self) -> Option<&Node> {
        self.node.as_ref()
    }
}

impl fmt::Display for End {
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
pub(crate) struct Ends {
    takes: Vec<End>,
    gives: End,
    implementer: Option<End>,
}

impl Ends {
    pub(crate) fn new(takes: Vec<End>, gives: End) -> Ends {
        Ends { takes, gives, implementer: None }
    }

    pub(crate) fn implemented_by(self, implementer: Option<End>) -> Ends {
        Ends { implementer, ..self }
    }

    pub(crate) fn all(&self) -> impl Iterator<Item = &End> {
        self.takes.iter().chain([&self.gives])
    }

    pub(crate) fn hop(&self) -> Option<Hop> {
        let takes: BTreeSet<Node> = self.taken().cloned().collect();
        let gives = self.gives.node.clone()?;
        (!takes.is_empty()).then_some(Hop { takes: Tuple(takes), gives })
    }

    pub(crate) fn taken(&self) -> impl Iterator<Item = &Node> {
        self.takes.iter().filter_map(|taken| taken.node.as_ref())
    }

    pub(crate) fn entered(&self, ground: Ground) -> Option<&Node> {
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

    pub(crate) fn key(&self) -> Option<Key> {
        let mut takes: Vec<Node> = self.takes.iter().map(|taken| taken.node.clone()).collect::<Option<Vec<Node>>>()?;
        let gives = self.gives.node.clone()?;
        let selector = self
            .implementer
            .as_ref()
            .and_then(|implementer| implementer.node.clone())
            .filter(|implementer| *implementer != gives && !takes.contains(implementer));
        takes.extend(selector);
        takes.sort();
        (!takes.is_empty()).then_some(Key { takes, gives })
    }

    pub(crate) fn applied(&self) -> Applied<'_> {
        Applied(self)
    }
}

impl AsRef<End> for Ends {
    fn as_ref(&self) -> &End {
        &self.gives
    }
}

#[derive(Debug)]
pub(crate) struct Applied<'ends>(&'ends Ends);

impl fmt::Display for Applied<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let takes: Vec<String> = self.0.takes.iter().skip(1).map(ToString::to_string).collect();
        write!(formatter, "{} -> {}", takes.join(", "), self.0.gives)
    }
}

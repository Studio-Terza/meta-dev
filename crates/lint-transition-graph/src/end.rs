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
pub(crate) struct Step {
    from: Node,
    to: Node,
}

impl Step {
    pub(crate) fn reached(&self) -> &Node {
        &self.to
    }
}

impl AsRef<Node> for Step {
    fn as_ref(&self) -> &Node {
        &self.from
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
}

impl Ends {
    pub(crate) fn new(takes: Vec<End>, gives: End) -> Ends {
        Ends { takes, gives }
    }

    pub(crate) fn all(&self) -> impl Iterator<Item = &End> {
        self.takes.iter().chain([&self.gives])
    }

    pub(crate) fn edges(&self) -> Vec<Key> {
        self.steps().into_iter().map(|step| Key { takes: vec![step.from], gives: step.to }).collect()
    }

    pub(crate) fn steps(&self) -> Vec<Step> {
        self.gives.node.as_ref().map_or_else(Vec::new, |to| {
            self.takes
                .iter()
                .filter_map(|taken| taken.node.as_ref())
                .map(|from| Step { from: from.clone(), to: to.clone() })
                .collect()
        })
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
        takes.sort();
        let gives = self.gives.node.clone()?;
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

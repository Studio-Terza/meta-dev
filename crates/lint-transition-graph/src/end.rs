use std::collections::BTreeSet;
use std::fmt;

use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Node {
    krate: CrateName,
    name: Name,
}

impl Node {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum End {
    Domain(Node),
    Primitive,
    Foreign,
}

impl End {
    fn domain(&self) -> Option<&Node> {
        match self {
            End::Domain(node) => Some(node),
            End::Primitive | End::Foreign => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Class {
    Covered,
    Primitive,
    Foreign,
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

    pub(crate) fn edges(&self) -> Vec<Key> {
        self.gives.domain().map_or_else(Vec::new, |to| {
            self.takes
                .iter()
                .filter_map(End::domain)
                .map(|from| Key { takes: vec![from.clone()], gives: to.clone() })
                .collect()
        })
    }

    pub(crate) fn class(&self) -> Class {
        let ends: Vec<&End> = self.takes.iter().chain([&self.gives]).collect();
        if ends.contains(&&End::Primitive) {
            Class::Primitive
        } else if ends.contains(&&End::Foreign) {
            Class::Foreign
        } else {
            Class::Covered
        }
    }

    pub(crate) fn key(&self) -> Option<Key> {
        let mut takes: Vec<Node> =
            self.takes.iter().map(|taken| taken.domain().cloned()).collect::<Option<Vec<Node>>>()?;
        takes.sort();
        let gives = self.gives.domain()?.clone();
        (!takes.is_empty()).then_some(Key { takes, gives })
    }
}

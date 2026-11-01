mod node_index;
mod referenced_names;
mod scope;

pub use node_index::NodeIndex;

use std::collections::{BTreeMap, BTreeSet};

use lint_core::Name;
use lint_core::SourceFile;
use referenced_names::ReferencedNames;
use scope::Scope;

#[derive(Clone, Debug, Default)]
pub struct ItemGraph {
    names: Vec<Name>,
    links: Vec<BTreeSet<NodeIndex>>,
}

impl ItemGraph {
    const EDGE_LENGTH: usize = 2;

    pub fn of_files(files: &[&SourceFile]) -> ItemGraph {
        let references: BTreeMap<Name, Vec<Name>> = files
            .iter()
            .flat_map(|file| ReferencedNames::of_items(&file.syntax().items, &Scope::of_file(file.syntax())).into_map())
            .fold(BTreeMap::new(), |mut all, (owner, uses)| {
                all.entry(owner).or_default().extend(uses);
                all
            });
        Self::of_references(&references)
    }

    pub fn of_references(references: &BTreeMap<Name, Vec<Name>>) -> ItemGraph {
        let index_of: BTreeMap<&Name, NodeIndex> =
            references.keys().enumerate().map(|(index, name)| (name, NodeIndex::from(index))).collect();
        let links = references
            .iter()
            .map(|(owner, uses)| {
                uses.iter().filter(|name| *name != owner).filter_map(|name| index_of.get(name).copied()).collect()
            })
            .collect();
        ItemGraph { names: references.keys().cloned().collect(), links }
    }

    pub fn edges(&self) -> Vec<[NodeIndex; Self::EDGE_LENGTH]> {
        self.links
            .iter()
            .enumerate()
            .flat_map(|(from, links)| links.iter().map(move |to| [NodeIndex::from(from), *to]))
            .collect()
    }

    pub fn name_at(&self, node: NodeIndex) -> Option<&Name> {
        self.names.get(node.index())
    }

    pub fn tangles(&self) -> Vec<Vec<NodeIndex>> {
        let reach: Vec<BTreeSet<NodeIndex>> =
            (0..self.names.len()).map(|index| self.reached(NodeIndex::from(index), BTreeSet::new())).collect();
        let tangles: BTreeSet<Vec<NodeIndex>> =
            (0..self.names.len()).map(|index| Self::tangle_at_index(&reach, NodeIndex::from(index))).collect();
        let mut tangles: Vec<Vec<NodeIndex>> = tangles.into_iter().collect();
        tangles.sort_by_key(|tangle| std::cmp::Reverse(tangle.len()));
        tangles
    }

    fn tangle_at_index(reach: &[BTreeSet<NodeIndex>], node: NodeIndex) -> Vec<NodeIndex> {
        let back = |other: &NodeIndex| reach.get(other.index()).is_some_and(|theirs| theirs.contains(&node));
        let forward = reach.get(node.index()).into_iter().flatten().copied();
        forward.filter(back).chain([node]).collect::<BTreeSet<NodeIndex>>().into_iter().collect()
    }

    fn reached(&self, node: NodeIndex, seen: BTreeSet<NodeIndex>) -> BTreeSet<NodeIndex> {
        let next: Vec<NodeIndex> = self.links.get(node.index()).into_iter().flatten().copied().collect();
        next.into_iter().fold(seen, |seen, next| self.grown(seen, next))
    }

    fn grown(&self, seen: BTreeSet<NodeIndex>, next: NodeIndex) -> BTreeSet<NodeIndex> {
        if seen.contains(&next) {
            seen
        } else {
            let mut grown = seen;
            grown.insert(next);
            self.reached(next, grown)
        }
    }
}

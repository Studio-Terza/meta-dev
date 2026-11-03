mod node_index;
mod referenced_names;
mod scope;
mod traversal;

pub use node_index::NodeIndex;
pub use traversal::Traversal;

use std::collections::{BTreeMap, BTreeSet};

use lint_core::SourceFile;
use lint_layout::Name;
use referenced_names::ReferencedNames;
use scope::Scope;

#[derive(Clone, Debug, Default)]
pub struct ItemGraph {
    names: Vec<Name>,
    links: Vec<Vec<NodeIndex>>,
}

impl ItemGraph {
    #[must_use]
    pub fn of_files(files: &[&SourceFile]) -> ItemGraph {
        let references = files
            .iter()
            .map(|file| ReferencedNames::of_items(&file.syntax().items, &Scope::of_file(file.syntax())))
            .fold(ReferencedNames::default(), ReferencedNames::merged);
        Self::of_references(&references.into_map())
    }

    #[must_use]
    pub fn of_references(references: &BTreeMap<Name, Vec<Name>>) -> ItemGraph {
        let index_of: BTreeMap<&Name, NodeIndex> =
            references.keys().enumerate().map(|(index, name)| (name, NodeIndex::from(index))).collect();
        let links = references
            .iter()
            .map(|(owner, uses)| {
                let linked: BTreeSet<NodeIndex> =
                    uses.iter().filter(|name| *name != owner).filter_map(|name| index_of.get(name).copied()).collect();
                linked.into_iter().collect()
            })
            .collect();
        ItemGraph { names: references.keys().cloned().collect(), links }
    }

    #[must_use]
    pub fn links(&self) -> &[Vec<NodeIndex>] {
        &self.links
    }

    #[must_use]
    pub fn name_at(&self, node: NodeIndex) -> Option<&Name> {
        self.names.get(node.into_inner())
    }

    #[must_use]
    pub fn tangles(&self) -> Vec<Vec<NodeIndex>> {
        let everyone = (0..self.names.len()).map(NodeIndex::from);
        let forward = |node: NodeIndex| self.links.get(node.into_inner()).into_iter().flatten().copied();
        let finished = Traversal::trees(everyone, forward).into_iter().flatten().rev();
        let backward = Self::reversed(&self.links);
        let tangles: BTreeSet<Vec<NodeIndex>> = Traversal::trees(finished, |node: NodeIndex| {
            backward.get(node.into_inner()).into_iter().flatten().copied()
        })
        .into_iter()
        .map(|component| component.into_iter().collect::<BTreeSet<NodeIndex>>().into_iter().collect())
        .collect();
        let mut tangles: Vec<Vec<NodeIndex>> = tangles.into_iter().collect();
        tangles.sort_by_key(|tangle| std::cmp::Reverse(tangle.len()));
        tangles
    }

    fn reversed(links: &[Vec<NodeIndex>]) -> Vec<Vec<NodeIndex>> {
        let edges =
            links.iter().enumerate().flat_map(|(from, to)| to.iter().map(move |to| (NodeIndex::from(from), *to)));
        edges.fold(vec![Vec::new(); links.len()], |mut backward, (from, to)| {
            backward.get_mut(to.into_inner()).into_iter().for_each(|reaching| reaching.push(from));
            backward
        })
    }
}

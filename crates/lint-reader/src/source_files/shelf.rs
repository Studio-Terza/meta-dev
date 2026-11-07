use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use file_ports::Files;
use lint_core::SourceFile;
use lint_layout::Failure;
use lint_placement::Packages;
use lint_roots::{Fact, Survey};

mod frontier;
mod store;

use super::located::{Located, Node};
use super::modules::{Child, Modules};
use super::scope::Scope;
use frontier::Frontier;
use store::Shelved;

#[derive(Debug)]
pub struct Shelf<'tree> {
    survey: Survey,
    packages: &'tree Packages,
    shelved: BTreeMap<Located<'tree>, Shelved>,
}

impl<'tree> Shelf<'tree> {
    #[must_use]
    pub fn stocked(files: &impl Files, packages: &'tree Packages, starts: &[Node<'tree>]) -> Self {
        Shelf { survey: Survey::default(), packages, shelved: BTreeMap::new() }.explored(files, starts, BTreeSet::new())
    }

    #[must_use]
    pub const fn survey(&self) -> &Survey {
        &self.survey
    }

    pub fn source(&mut self, scope: &Scope<'tree>, file: &Path) -> Option<Result<SourceFile, Failure>> {
        self.shelved.get_mut(&Located::new(scope.tree(), scope.package(), file)).and_then(Shelved::take_parsed)
    }

    #[must_use]
    pub fn test_source(&self, scope: &Scope<'tree>, file: &Path) -> Option<SourceFile> {
        let located = Located::new(scope.tree(), scope.package(), file);
        self.shelved.get(&located).and_then(Shelved::opened).and_then(|opened| opened.parsed().ok())
    }

    pub fn explored(self, files: &impl Files, level: &[Node<'tree>], visited: BTreeSet<Node<'tree>>) -> Self {
        let (_, fresh): (Vec<&Located<'tree>>, Vec<&Located<'tree>>) =
            level.iter().map(Node::located).partition(|located| self.shelved.contains_key(*located));
        let fresh: BTreeSet<Located<'tree>> = fresh.into_iter().cloned().collect();
        let stocked = self.read(files, fresh.into_iter().collect());
        let candidates: Vec<PathBuf> = level.iter().flat_map(|node| stocked.candidates(node)).collect();
        let stocked = Shelf { survey: stocked.survey.learn(files, Fact::Followed, &candidates), ..stocked };
        let visited: BTreeSet<Node<'tree>> = visited.into_iter().chain(level.iter().cloned()).collect();
        let reached: BTreeSet<Node<'tree>> = level
            .iter()
            .flat_map(|node| stocked.declared(node).into_iter().map(move |child| (node, child)))
            .map(|(node, child)| child.canonical(&stocked.survey).visit(|file, children| node.child(file, children)))
            .collect();
        let (_, next): (Vec<Node<'tree>>, Vec<Node<'tree>>) =
            reached.into_iter().partition(|node| visited.contains(node));
        Frontier::new(next, visited).explored(stocked, files)
    }

    fn read(self, files: &impl Files, fresh: Vec<Located<'tree>>) -> Self {
        let opened = Located::open_many(files, self.packages, &fresh);
        fresh.into_iter().zip(opened).fold(self, |mut shelf, (located, opened)| {
            shelf.shelved.insert(located, Shelved::from(opened));
            shelf
        })
    }

    fn candidates(&self, node: &Node<'tree>) -> Vec<PathBuf> {
        self.shelved
            .get(node.located())
            .and_then(Shelved::parsed)
            .into_iter()
            .flat_map(|parsed| Modules::candidates(node.located().file(), node.children(), &parsed.syntax().items))
            .collect()
    }

    fn declared(&self, node: &Node<'tree>) -> Vec<Child> {
        self.shelved
            .get(node.located())
            .and_then(Shelved::parsed)
            .into_iter()
            .flat_map(|parsed| {
                Modules::nested_in(&self.survey, node.located().file(), node.children(), &parsed.syntax().items)
            })
            .collect()
    }
}

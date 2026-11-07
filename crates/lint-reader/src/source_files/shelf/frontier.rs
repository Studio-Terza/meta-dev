use std::collections::BTreeSet;

use file_ports::Files;

use super::Shelf;
use crate::source_files::located::Node;

#[derive(Debug)]
pub struct Frontier<'tree> {
    next: Vec<Node<'tree>>,
    visited: BTreeSet<Node<'tree>>,
}

impl<'tree> Frontier<'tree> {
    #[must_use]
    pub const fn new(next: Vec<Node<'tree>>, visited: BTreeSet<Node<'tree>>) -> Self {
        Frontier { next, visited }
    }

    #[must_use]
    pub fn explored(self, stocked: Shelf<'tree>, files: &impl Files) -> Shelf<'tree> {
        match self.next.first() {
            Some(_) => stocked.explored(files, &self.next, self.visited),
            None => stocked,
        }
    }
}

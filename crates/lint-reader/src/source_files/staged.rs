use std::path::Path;

use file_ports::Files;
use lint_layout::Failure;
use lint_roots::{Roots, Target, Tree};

use super::located::{Home, Node};

#[derive(Debug)]
pub struct Staged {
    targets: Vec<Target>,
}

impl Staged {
    pub fn under(files: &(impl Files + Sync), trees: &[&Tree<'_>]) -> Vec<Result<Staged, Failure>> {
        let below = Roots::surveyed(files, trees);
        trees
            .iter()
            .zip(Roots::under(files, trees, below))
            .map(|(tree, rooted)| {
                let found = rooted?;
                found.first().ok_or_else(|| tree.without_sources())?;
                Ok(Staged { targets: found })
            })
            .collect()
    }

    #[must_use]
    pub fn starts<'tree>(&self, tree: &'tree Tree<'tree>) -> Vec<Node<'tree>> {
        self.targets
            .iter()
            .flat_map(|target| {
                target.files().iter().map(move |file| {
                    let children = file.parent().map_or_default(Path::to_path_buf);
                    Home::new(tree, target.package()).located(file).node(&children)
                })
            })
            .collect()
    }

    #[must_use]
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }
}

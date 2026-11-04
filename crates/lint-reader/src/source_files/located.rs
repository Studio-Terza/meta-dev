use std::path::{Path, PathBuf};

use file_ports::{FileFailure, Files, Text};
use lint_core::SourceText;
use lint_layout::Failure;
use lint_placement::Placement;
use lint_roots::Tree;
use newtype::Newtype;

use super::opened::Opened;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Located<'tree> {
    tree: &'tree Tree<'tree>,
    package: PathBuf,
    file: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node<'tree> {
    located: Located<'tree>,
    children: PathBuf,
}

impl<'tree> Located<'tree> {
    #[must_use]
    pub fn new(tree: &'tree Tree<'tree>, package: &Path, file: &Path) -> Located<'tree> {
        Located { tree, package: package.to_path_buf(), file: file.to_path_buf() }
    }

    pub fn open_many(files: &impl Files, fresh: &[Located<'tree>]) -> Vec<Result<Opened, Failure>> {
        let paths: Vec<&Path> = fresh.iter().map(|located| located.file.as_path()).collect();
        let packages: Vec<&Path> = fresh.iter().map(|located| located.package.as_path()).collect();
        fresh
            .iter()
            .zip(files.read_text_many(&paths))
            .zip(Placement::within_many(files, &packages, &paths))
            .map(|((located, text), placement)| located.opened(text, placement))
            .collect()
    }

    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }

    #[must_use]
    pub fn node(self, children: &Path) -> Node<'tree> {
        Node { located: self, children: children.to_path_buf() }
    }

    fn opened(&self, text: Result<Text, FileFailure>, placement: Placement) -> Result<Opened, Failure> {
        text.map_err(|failure| Failure::Io { path: self.tree.display(&self.file), source: failure.into() }).map(
            |text| {
                Opened::new(
                    self.tree.display(&self.file),
                    self.file.clone(),
                    SourceText::new(text.into_inner()),
                    placement,
                )
            },
        )
    }
}

impl<'tree> Node<'tree> {
    #[must_use]
    pub fn located(&self) -> &Located<'tree> {
        &self.located
    }

    #[must_use]
    pub fn children(&self) -> &Path {
        &self.children
    }

    #[must_use]
    pub fn child(&self, file: &Path, children: &Path) -> Node<'tree> {
        Located::new(self.located.tree, &self.located.package, file).node(children)
    }
}

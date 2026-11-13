use std::path::{Path, PathBuf};

use file_ports::{FileFailure, Files, Text};
use lint_core::SourceText;
use lint_layout::Failure;
use lint_placement::{Packages, Placement};
use lint_roots::{Survey, Tree};
use newtype::Wrapping;

use super::modules::Child;
use super::opened::Opened;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Located<'tree> {
    file: PathBuf,
    package: PathBuf,
    tree: &'tree Tree<'tree>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node<'tree> {
    located: Located<'tree>,
    children: PathBuf,
}

impl<'tree> Located<'tree> {
    #[must_use]
    pub fn new(tree: &'tree Tree<'tree>, package: &Path, file: &Path) -> Located<'tree> {
        Located { file: file.to_path_buf(), package: package.to_path_buf(), tree }
    }

    pub fn open_many(
        files: &impl Files,
        packages: &Packages,
        fresh: &[Located<'tree>],
    ) -> Vec<Result<Opened, Failure>> {
        let paths: Vec<&Path> = fresh.iter().map(|located| located.file.as_path()).collect();
        fresh
            .iter()
            .zip(files.read_text_many(&paths))
            .map(|(located, text)| located.opened(text, packages.placed(&located.package, &located.file)))
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
            |read| {
                Opened::new(
                    self.tree.display(&self.file),
                    self.file.clone(),
                    SourceText::new(read.into_inner()),
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
    pub fn child_of(&self, child: &Child, survey: &Survey) -> Node<'tree> {
        child.clone().canonical(survey).visit(|file, children| self.child(file, children))
    }

    #[must_use]
    pub fn child(&self, file: &Path, children: &Path) -> Node<'tree> {
        Located::new(self.located.tree, &self.located.package, file).node(children)
    }
}

use std::path::{Path, PathBuf};

use file_ports::{FileFailure, Files, Text};
use lint_core::{SourceFile, SourceText};
use lint_layout::Failure;
use lint_placement::{Packages, Placement};
use lint_roots::{Survey, Tree};
use newtype::Wrapping;

use super::modules::Child;

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
    pub fn parsed_many(
        files: &impl Files,
        packages: &Packages,
        fresh: &[Located<'tree>],
    ) -> Vec<Result<SourceFile, Failure>> {
        let paths: Vec<&Path> = fresh.iter().map(|located| located.file.as_path()).collect();
        fresh
            .iter()
            .zip(files.read_text_many(&paths))
            .map(|(located, text)| located.parsed(text, packages.placed(&located.package, &located.file)))
            .collect()
    }

    #[must_use]
    pub fn node(self, children: &Path) -> Node<'tree> {
        Node { located: self, children: children.to_path_buf() }
    }

    fn parsed(&self, text: Result<Text, FileFailure>, placement: Placement) -> Result<SourceFile, Failure> {
        let shown = self.tree.display(&self.file);
        text.map_err(|failure| Failure::Io { path: shown.clone(), source: failure.into() }).and_then(|read| {
            SourceFile::parse_at(shown, self.file.clone(), placement, SourceText::new(read.into_inner()))
        })
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
    pub fn home(&self) -> Home<'_, 'tree> {
        Home { tree: self.located.tree, package: &self.located.package }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Home<'node, 'tree> {
    tree: &'tree Tree<'tree>,
    package: &'node Path,
}

impl<'node, 'tree> Home<'node, 'tree> {
    #[must_use]
    pub fn new(tree: &'tree Tree<'tree>, package: &'node Path) -> Home<'node, 'tree> {
        Home { tree, package }
    }

    #[must_use]
    pub fn located(self, file: &Path) -> Located<'tree> {
        Located { file: file.to_path_buf(), package: self.package.to_path_buf(), tree: self.tree }
    }

    #[must_use]
    pub fn child(self, child: &Child, survey: &Survey) -> Node<'tree> {
        self.located(child.canonical(survey)).node(child.children())
    }
}

impl AsRef<Path> for Located<'_> {
    fn as_ref(&self) -> &Path {
        &self.file
    }
}

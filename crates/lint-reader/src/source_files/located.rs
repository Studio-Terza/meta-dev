use std::path::{Path, PathBuf};

use file_ports::{FileFailure, Files, Text};
use lint_core::{SourceFile, SourceText};
use lint_layout::Failure;
use lint_placement::Placement;
use lint_roots::Tree;
use newtype::Newtype;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Located<'tree> {
    tree: &'tree Tree,
    package: PathBuf,
    file: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node<'tree> {
    located: Located<'tree>,
    children: PathBuf,
}

#[derive(Debug)]
pub struct Opened {
    shown: PathBuf,
    file: PathBuf,
    text: SourceText,
    placement: Placement,
}

impl<'tree> Located<'tree> {
    #[must_use]
    pub fn new(tree: &'tree Tree, package: &Path, file: &Path) -> Located<'tree> {
        Located { tree, package: package.to_path_buf(), file: file.to_path_buf() }
    }

    pub fn open_many(files: &impl Files, fresh: &[Located<'tree>]) -> Vec<Result<Opened, Failure>> {
        let paths: Vec<PathBuf> = fresh.iter().map(|located| located.file.clone()).collect();
        let packages: Vec<PathBuf> = fresh.iter().map(|located| located.package.clone()).collect();
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
        let shown = self.tree.display(&self.file);
        match text {
            Ok(text) => {
                Ok(Opened { shown, file: self.file.clone(), text: SourceText::new(text.into_inner()), placement })
            }
            Err(failure) => Err(Failure::Io { path: shown, source: failure.into() }),
        }
    }
}

impl Opened {
    pub fn parsed(&self) -> Result<SourceFile, Failure> {
        SourceFile::parse_at(self.shown.clone(), self.file.clone(), self.placement.clone(), self.text.clone())
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

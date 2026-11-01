mod placement;

use std::path::{Path, PathBuf};

use crate::failure::Failure;
use crate::line_number::LineNumber;
use crate::role::Role;
use placement::Placement;

#[derive(Debug)]
pub struct SourceFile {
    path: PathBuf,
    placement: Placement,
    text: String,
    syntax: syn::File,
    test_modules: Vec<SourceFile>,
}

impl SourceFile {
    pub fn parse(path: impl Into<PathBuf>, text: String) -> Result<Self, Failure> {
        let path = path.into();
        let placement = Placement::locate(&path);
        Self::parse_placed(path, placement, text)
    }

    pub fn parse_at(path: PathBuf, package: &Path, located: &Path, text: String) -> Result<Self, Failure> {
        Self::parse_placed(path, Placement::within(package, located), text)
    }

    fn parse_placed(path: PathBuf, placement: Placement, text: String) -> Result<Self, Failure> {
        let syntax = syn::parse_file(&text).map_err(|source| Failure::Parse {
            path: path.clone(),
            line: LineNumber::from(source.span()),
            source,
        })?;
        Ok(Self { path, placement, text, syntax, test_modules: Vec::new() })
    }

    #[must_use]
    pub fn with_test_modules(self, test_modules: Vec<SourceFile>) -> SourceFile {
        SourceFile { test_modules, ..self }
    }

    #[must_use]
    pub fn test_modules(&self) -> &[SourceFile] {
        &self.test_modules
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn placement(&self) -> &Placement {
        &self.placement
    }

    #[must_use]
    pub fn is_shipped(&self) -> bool {
        self.placement.role() != Role::Target
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn syntax(&self) -> &syn::File {
        &self.syntax
    }
}

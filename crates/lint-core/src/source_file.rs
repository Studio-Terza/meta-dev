use std::path::{Path, PathBuf};

use crate::failure::Failure;
use crate::line_number::LineNumber;
use crate::module_path::ModulePath;
use crate::placement::Placement;
use crate::role::Role;

#[derive(Debug)]
pub struct SourceFile {
    path: PathBuf,
    placement: Placement,
    text: String,
    syntax: syn::File,
}

impl SourceFile {
    pub fn parse(path: impl Into<PathBuf>, text: String) -> Result<Self, Failure> {
        let path = path.into();
        let located = path.clone();
        Self::parse_at(path, &located, text)
    }

    pub fn parse_at(path: PathBuf, located: &Path, text: String) -> Result<Self, Failure> {
        let syntax = syn::parse_file(&text).map_err(|source| Failure::Parse {
            path: path.clone(),
            line: LineNumber::from(source.span()),
            source,
        })?;
        Ok(Self { path, placement: Placement::of(located), text, syntax })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn crate_root(&self) -> &Path {
        self.placement.crate_root()
    }

    #[must_use]
    pub fn module(&self) -> &ModulePath {
        self.placement.module()
    }

    #[must_use]
    pub fn role(&self) -> Role {
        self.placement.role()
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

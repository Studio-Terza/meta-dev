use std::path::{Path, PathBuf};

use crate::failure::Failure;

#[derive(Debug)]
pub struct SourceFile {
    path: PathBuf,
    text: String,
    syntax: syn::File,
}

impl SourceFile {
    pub fn parse(path: impl Into<PathBuf>, text: String) -> Result<Self, Failure> {
        let path = path.into();
        let syntax = syn::parse_file(&text).map_err(|source| Failure::Parse { path: path.clone(), source })?;
        Ok(Self { path, text, syntax })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn crate_root(&self) -> &Path {
        self.path
            .ancestors()
            .find(|ancestor| ancestor.file_name().is_some_and(|name| name == "src"))
            .and_then(Path::parent)
            .unwrap_or(Path::new(""))
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

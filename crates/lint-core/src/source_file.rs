mod placement;

use std::path::{Path, PathBuf};

use crate::crate_file::CrateFile;
use crate::failure::Failure;
use crate::line_number::LineNumber;
use crate::role::Role;
use placement::Placement;

#[derive(Debug)]
pub struct SourceFile {
    path: PathBuf,
    located: PathBuf,
    placement: Placement,
    crate_file: CrateFile,
    text: String,
    syntax: syn::File,
    test_modules: Vec<SourceFile>,
}

impl SourceFile {
    const MANIFEST: &'static str = "Cargo.toml";

    pub fn parse(path: impl Into<PathBuf>, text: String) -> Result<Self, Failure> {
        let path = path.into();
        let placement = Placement::locate(&path);
        let located = path.clone();
        Self::parse_placed(path, located, placement, text)
    }

    pub fn parse_at(path: PathBuf, package: &Path, located: &Path, text: String) -> Result<Self, Failure> {
        Self::parse_placed(path, located.to_path_buf(), Placement::within(package, located), text)
    }

    fn parse_placed(path: PathBuf, located: PathBuf, placement: Placement, text: String) -> Result<Self, Failure> {
        let syntax = syn::parse_file(&text).map_err(|source| Failure::Parse {
            path: path.clone(),
            line: LineNumber::from(source.span()),
            source,
        })?;
        let crate_file = placement.crate_file(&path);
        Ok(Self { path, located, placement, crate_file, text, syntax, test_modules: Vec::new() })
    }

    #[must_use]
    pub fn with_test_modules(self, test_modules: Vec<SourceFile>) -> SourceFile {
        SourceFile { test_modules, ..self }
    }

    #[must_use]
    pub fn in_crate_file(self, crate_file: CrateFile) -> SourceFile {
        SourceFile { crate_file, ..self }
    }

    #[must_use]
    pub fn crate_file(&self) -> &CrateFile {
        &self.crate_file
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
    pub fn switched_off(&self) -> Option<&Path> {
        self.placement.package().untested().holds(&self.located).then_some(self.path.as_path())
    }

    #[must_use]
    pub fn manifest_file(&self) -> PathBuf {
        let inside = self.located.strip_prefix(self.placement.package().root().directory()).ok();
        let depth = inside.map_or(0, |inside| inside.components().count());
        self.path.ancestors().nth(depth).unwrap_or(&self.path).join(Self::MANIFEST)
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

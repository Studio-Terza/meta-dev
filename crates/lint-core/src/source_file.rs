mod boundary;
mod located;
mod manifest_file;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use boundary::Nesting;
use file_ports::Absent;
use lint_layout::Failure;
use lint_layout::LineNumber;
use lint_layout::SourceText;
use lint_placement::CrateFile;
use lint_placement::Placement;
use lint_placement::Role;
use located::Located;
pub use manifest_file::ManifestFile;

#[derive(Debug)]
pub struct SourceFile {
    path: PathBuf,
    located: Located,
    placement: Placement,
    crate_file: CrateFile,
    text: SourceText,
    syntax: syn::File,
    test_modules: Vec<SourceFile>,
    manifest: OnceLock<ManifestFile>,
}

impl SourceFile {
    const MANIFEST: &'static str = "Cargo.toml";

    pub fn parse(path: impl Into<PathBuf>, text: SourceText) -> Result<Self, Failure> {
        let path = path.into();
        let placement = Placement::locate(&Absent, &path);
        Self::parsed(path, Located::Shown, placement, text)
    }

    pub fn parse_at(path: PathBuf, located: PathBuf, placement: Placement, text: SourceText) -> Result<Self, Failure> {
        Self::parsed(path, Located::At(located), placement, text)
    }

    fn parsed(path: PathBuf, located: Located, placement: Placement, text: SourceText) -> Result<Self, Failure> {
        let parsed = Nesting::within_limit(&text)
            .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), Nesting))
            .and_then(|text| syn::parse_file(text.as_ref()));
        match parsed {
            Ok(syntax) => Ok(Self {
                crate_file: placement.crate_file(&path),
                path,
                located,
                placement,
                text,
                syntax,
                test_modules: Vec::new(),
                manifest: OnceLock::new(),
            }),
            Err(source) => Err(Failure::Parse { path, line: LineNumber::from(source.span()), source }),
        }
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
        self.placement.package().untested().holds(self.located.within(&self.path)).map(|_| self.path.as_path())
    }

    #[must_use]
    pub fn manifest_file(&self) -> &ManifestFile {
        self.manifest.get_or_init(|| {
            let inside = self.located.within(&self.path).strip_prefix(self.placement.package().root().directory()).ok();
            let depth = inside.map_or(0, |inside| inside.components().count());
            ManifestFile::from(self.path.ancestors().nth(depth).unwrap_or(&self.path).join(Self::MANIFEST))
        })
    }

    #[must_use]
    pub fn placement(&self) -> &Placement {
        &self.placement
    }

    #[must_use]
    pub fn shipped(&self) -> Option<&SourceFile> {
        (self.placement.role() != Role::Target).then_some(self)
    }

    #[must_use]
    pub fn text(&self) -> &SourceText {
        &self.text
    }

    #[must_use]
    pub fn syntax(&self) -> &syn::File {
        &self.syntax
    }
}

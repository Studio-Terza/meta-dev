mod boundary;
mod head;
mod located;

use std::path::PathBuf;

use crate::macros::Expansions;
use boundary::Nesting;
use file_ports::Absent;
pub use head::FileHead;
use lint_layout::Failure;
use lint_layout::LineNumber;
use lint_layout::SourceText;
use lint_placement::CrateFile;
use lint_placement::Placement;
use located::Located;

#[derive(Debug)]
pub struct SourceFile {
    head: FileHead,
    text: SourceText,
    syntax: syn::File,
    test_modules: Vec<SourceFile>,
    expansions: Expansions,
}

impl SourceFile {
    pub fn parse(path: impl Into<PathBuf>, text: SourceText) -> Result<Self, Failure> {
        let file_path = path.into();
        let placement = Placement::locate(&Absent, &file_path);
        Self::parsed(file_path, Located::Shown, placement, text)
    }

    pub fn parse_at(path: PathBuf, located: PathBuf, placement: Placement, text: SourceText) -> Result<Self, Failure> {
        Self::parsed(path, Located::At(located), placement, text)
    }

    fn parsed(path: PathBuf, located: Located, placement: Placement, text: SourceText) -> Result<Self, Failure> {
        match Nesting::within_limit(&text)
            .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), Nesting))
            .and_then(|bounded| syn::parse_file(bounded.as_ref()))
        {
            Ok(syntax) => Ok(Self {
                head: FileHead::new(path, located, placement),
                text,
                syntax,
                test_modules: Vec::new(),
                expansions: Expansions::default(),
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
        SourceFile { head: self.head.in_crate_file(crate_file), ..self }
    }

    #[must_use]
    pub fn head(&self) -> &FileHead {
        &self.head
    }

    #[must_use]
    pub fn test_modules(&self) -> &[SourceFile] {
        &self.test_modules
    }

    #[must_use]
    pub fn shipped(&self) -> Option<&SourceFile> {
        self.head.shipped().map(|_| self)
    }

    #[must_use]
    pub fn text(&self) -> &SourceText {
        &self.text
    }

    #[must_use]
    pub fn syntax(&self) -> &syn::File {
        &self.syntax
    }

    #[must_use]
    pub(crate) fn expansions(&self) -> &Expansions {
        &self.expansions
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Parsed<'syntax, 'head> {
    file: &'syntax SourceFile,
    head: &'head FileHead,
}

impl<'syntax, 'head> Parsed<'syntax, 'head> {
    #[must_use]
    pub fn new(file: &'syntax SourceFile, head: &'head FileHead) -> Parsed<'syntax, 'head> {
        Parsed { file, head }
    }

    #[must_use]
    pub fn file(self) -> &'syntax SourceFile {
        self.file
    }

    #[must_use]
    pub fn head(self) -> &'head FileHead {
        self.head
    }
}

impl<'file> From<&'file SourceFile> for Parsed<'file, 'file> {
    fn from(file: &'file SourceFile) -> Parsed<'file, 'file> {
        Parsed { file, head: &file.head }
    }
}

use crate::source_file::{FileHead, SourceFile};
use lint_layout::Name;
use lint_placement::CrateFile;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Module<'file> {
    crate_file: &'file CrateFile,
    path: Vec<Name>,
}

impl<'file> Module<'file> {
    pub const UNPLACED: Module<'static> = Module { crate_file: &CrateFile::UNPLACED, path: Vec::new() };

    #[must_use]
    pub fn of_file(file: &'file SourceFile) -> Module<'file> {
        Module::of_head(file.head())
    }

    #[must_use]
    pub fn of_head(head: &'file FileHead) -> Module<'file> {
        Module { crate_file: head.crate_file(), path: AsRef::<[Name]>::as_ref(head.placement().module()).to_vec() }
    }

    #[must_use]
    pub fn within(&self, name: Name) -> Module<'file> {
        Module { crate_file: self.crate_file, path: self.path.iter().cloned().chain(Some(name)).collect() }
    }
}

impl<'file> From<&Module<'file>> for &'file CrateFile {
    fn from(module: &Module<'file>) -> &'file CrateFile {
        module.crate_file
    }
}

impl<'file> From<&'file CrateFile> for Module<'file> {
    fn from(crate_file: &'file CrateFile) -> Module<'file> {
        Module { crate_file, path: Vec::new() }
    }
}

use lint_core::{FileHead, SourceFile};
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
        Module { crate_file: head.crate_file(), path: head.placement().module().segments().to_vec() }
    }

    #[must_use]
    pub fn within(&self, name: Name) -> Module<'file> {
        Module { crate_file: self.crate_file, path: self.path.iter().cloned().chain(Some(name)).collect() }
    }

    pub fn enter(&mut self, name: Name) {
        self.path.push(name);
    }

    pub fn leave(&mut self) {
        self.path.pop();
    }

    pub(crate) fn parent(&self) -> Option<Module<'file>> {
        let outer = self.path.len().checked_sub(1)?;
        Some(Module { crate_file: self.crate_file, path: self.path.iter().take(outer).cloned().collect() })
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

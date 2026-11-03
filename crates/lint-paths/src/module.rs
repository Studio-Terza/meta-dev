use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateFile;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Module<'file> {
    crate_file: &'file CrateFile,
    path: Vec<Name>,
}

static UNPLACED: CrateFile = CrateFile::UNPLACED;

impl<'file> Module<'file> {
    #[must_use]
    pub fn of_file(file: &'file SourceFile) -> Module<'file> {
        Module { crate_file: file.crate_file(), path: file.placement().module().segments().to_vec() }
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

    #[must_use]
    pub fn root(&self) -> Module<'file> {
        Module { crate_file: self.crate_file, path: Vec::new() }
    }

    pub(crate) fn ancestry(&self) -> Vec<Module<'file>> {
        (0..=self.path.len())
            .map(|length| Module {
                crate_file: self.crate_file,
                path: self.path.iter().take(length).cloned().collect(),
            })
            .collect()
    }
}

impl Default for Module<'_> {
    fn default() -> Self {
        Module { crate_file: &UNPLACED, path: Vec::new() }
    }
}

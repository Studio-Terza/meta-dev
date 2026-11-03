use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateFile;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Module {
    crate_file: CrateFile,
    path: Vec<Name>,
}

impl Module {
    #[must_use]
    pub fn of_file(file: &SourceFile) -> Module {
        Module { crate_file: file.crate_file().clone(), path: file.placement().module().segments().to_vec() }
    }

    #[must_use]
    pub fn within(&self, name: &Name) -> Module {
        Module {
            crate_file: self.crate_file.clone(),
            path: self.path.iter().cloned().chain(Some(name.clone())).collect(),
        }
    }

    pub(crate) fn parent(&self) -> Option<Module> {
        let outer = self.path.len().checked_sub(1)?;
        Some(Module { crate_file: self.crate_file.clone(), path: self.path.iter().take(outer).cloned().collect() })
    }
}

impl AsRef<CrateFile> for Module {
    fn as_ref(&self) -> &CrateFile {
        &self.crate_file
    }
}

impl From<&CrateFile> for Module {
    fn from(crate_file: &CrateFile) -> Module {
        Module { crate_file: crate_file.clone(), path: Vec::new() }
    }
}

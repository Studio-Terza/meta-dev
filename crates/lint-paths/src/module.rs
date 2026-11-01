use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateFile;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Module {
    crate_file: CrateFile,
    path: Vec<Name>,
}

impl Module {
    pub(crate) fn of_file(file: &SourceFile) -> Module {
        Module { crate_file: file.crate_file().clone(), path: file.placement().module().segments().to_vec() }
    }

    pub(crate) fn within(&self, name: &Name) -> Module {
        Module {
            crate_file: self.crate_file.clone(),
            path: self.path.iter().cloned().chain(Some(name.clone())).collect(),
        }
    }

    pub(crate) fn parent(&self) -> Option<Module> {
        let outer = self.path.len().checked_sub(1)?;
        Some(Module { crate_file: self.crate_file.clone(), path: self.path.iter().take(outer).cloned().collect() })
    }

    pub(crate) fn root(&self) -> Module {
        Module { crate_file: self.crate_file.clone(), path: Vec::new() }
    }

    pub(crate) fn ancestry(&self) -> Vec<Module> {
        (0..=self.path.len())
            .map(|length| Module {
                crate_file: self.crate_file.clone(),
                path: self.path.iter().take(length).cloned().collect(),
            })
            .collect()
    }
}

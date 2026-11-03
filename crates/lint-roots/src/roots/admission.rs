mod workspace;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use file_ports::Files;
use lint_layout::Failure;
use lint_layout::Manifest;
use workspace::Workspace;

#[derive(Debug, Default)]
pub struct Admission {
    workspaces: BTreeMap<PathBuf, Workspace>,
    admitted: Vec<Manifest>,
}

impl Admission {
    pub fn members_among(files: &impl Files, packages: Vec<Manifest>) -> Result<Vec<Manifest>, Failure> {
        packages
            .into_iter()
            .try_fold(Admission::default(), |admission, package| admission.considering(files, package))
            .map(|admission| admission.admitted)
    }

    fn considering(self, files: &impl Files, package: Manifest) -> Result<Admission, Failure> {
        match Self::enclosing(files, package.directory())? {
            Some(root) if package.workspace().is_none() => self.within(files, &root, package),
            _ => Ok(self.admitting(Some(package))),
        }
    }

    fn within(self, files: &impl Files, root: &Path, package: Manifest) -> Result<Admission, Failure> {
        let workspaces = if self.workspaces.contains_key(root) {
            self.workspaces
        } else {
            let loaded = Workspace::load(files, root)?;
            self.workspaces.into_iter().chain([(root.to_path_buf(), loaded)]).collect()
        };
        let member = Some(package).filter(|candidate| {
            workspaces.get(root).and_then(|workspace| workspace.admits(candidate.directory())).is_some()
        });
        Ok(Admission { workspaces, admitted: self.admitted }.admitting(member))
    }

    fn admitting(self, package: Option<Manifest>) -> Admission {
        Admission { admitted: self.admitted.into_iter().chain(package).collect(), ..self }
    }

    fn enclosing(files: &impl Files, directory: &Path) -> Result<Option<PathBuf>, Failure> {
        directory
            .ancestors()
            .skip(1)
            .map(|ancestor| Manifest::read(files, ancestor))
            .find_map(|read| match read {
                Ok(Some(manifest)) if manifest.workspace().is_some() => Some(Ok(manifest.directory().to_path_buf())),
                Ok(_) => None,
                Err(failure) => Some(Err(failure)),
            })
            .transpose()
    }
}

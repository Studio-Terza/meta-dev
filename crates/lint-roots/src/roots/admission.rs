mod workspace;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use lint_layout::Failure;
use lint_layout::Manifest;
use workspace::Workspace;

#[derive(Debug, Default)]
pub struct Admission {
    workspaces: BTreeMap<PathBuf, Workspace>,
    admitted: Vec<Manifest>,
}

impl Admission {
    pub fn members_among(packages: Vec<Manifest>) -> Result<Vec<Manifest>, Failure> {
        packages.into_iter().try_fold(Admission::default(), Admission::considering).map(|admission| admission.admitted)
    }

    fn considering(self, package: Manifest) -> Result<Admission, Failure> {
        match Self::enclosing(package.directory())? {
            Some(root) if package.workspace().is_none() => self.within(&root, package),
            _ => Ok(self.admitting(Some(package))),
        }
    }

    fn within(self, root: &Path, package: Manifest) -> Result<Admission, Failure> {
        let workspaces = if self.workspaces.contains_key(root) {
            self.workspaces
        } else {
            let loaded = Workspace::load(root)?;
            self.workspaces.into_iter().chain([(root.to_path_buf(), loaded)]).collect()
        };
        let member = Some(package)
            .filter(|candidate| workspaces.get(root).is_some_and(|workspace| workspace.admits(candidate.directory())));
        Ok(Admission { workspaces, admitted: self.admitted }.admitting(member))
    }

    fn admitting(self, package: Option<Manifest>) -> Admission {
        Admission { admitted: self.admitted.into_iter().chain(package).collect(), ..self }
    }

    fn enclosing(directory: &Path) -> Result<Option<PathBuf>, Failure> {
        directory
            .ancestors()
            .skip(1)
            .map(Manifest::read)
            .find_map(|read| match read {
                Ok(Some(manifest)) if manifest.workspace().is_some() => Some(Ok(manifest.directory().to_path_buf())),
                Ok(_) => None,
                Err(failure) => Some(Err(failure)),
            })
            .transpose()
    }
}

mod workspace;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::regrouped::Regrouped;
use file_ports::Files;
use lint_layout::Failure;
use lint_layout::Manifest;
use workspace::Workspace;

#[derive(Debug, Default)]
pub struct Admission {
    loaded: BTreeMap<PathBuf, Result<Workspace, Failure>>,
    admitted: Vec<Manifest>,
}

impl Admission {
    pub fn members_among(
        files: &impl Files,
        groups: Vec<Result<Vec<Manifest>, Failure>>,
    ) -> Vec<Result<Vec<Manifest>, Failure>> {
        let roots: Vec<Vec<PathBuf>> =
            groups.iter().map(|group| group.as_ref().map_or_else(|_| Vec::new(), |found| Self::roots(found))).collect();
        let loaded = Regrouped::like(&roots, Workspace::load_many(files, &roots.concat()));
        groups
            .into_iter()
            .zip(roots.into_iter().zip(loaded))
            .map(|(group, (roots, loaded))| {
                group.and_then(|found| Self::admitted(found, roots.into_iter().zip(loaded).collect()))
            })
            .collect()
    }

    #[must_use]
    pub fn enclosing(workspaces: &BTreeSet<PathBuf>, directory: &Path) -> Option<PathBuf> {
        directory.ancestors().skip(1).find(|ancestor| workspaces.contains(*ancestor)).map(Path::to_path_buf)
    }

    fn roots(found: &[Manifest]) -> Vec<PathBuf> {
        let workspaces: BTreeSet<PathBuf> = found
            .iter()
            .filter(|manifest| manifest.workspace().is_some())
            .map(|manifest| manifest.directory().to_path_buf())
            .collect();
        let roots: BTreeSet<PathBuf> = found
            .iter()
            .filter(|manifest| manifest.package().is_some())
            .filter(|manifest| manifest.workspace().is_none())
            .filter_map(|package| Self::enclosing(&workspaces, package.directory()))
            .collect();
        roots.into_iter().collect()
    }

    fn admitted(
        found: Vec<Manifest>,
        loaded: BTreeMap<PathBuf, Result<Workspace, Failure>>,
    ) -> Result<Vec<Manifest>, Failure> {
        let workspaces: BTreeSet<PathBuf> = found
            .iter()
            .filter(|manifest| manifest.workspace().is_some())
            .map(|manifest| manifest.directory().to_path_buf())
            .collect();
        found
            .into_iter()
            .filter(|manifest| manifest.package().is_some())
            .try_fold(Admission { loaded, admitted: Vec::new() }, |admission, package| {
                match Self::enclosing(&workspaces, package.directory()) {
                    Some(root) if package.workspace().is_none() => admission.within(&root, package),
                    _ => Ok(admission.admitting(Some(package))),
                }
            })
            .map(|admission| admission.admitted)
    }

    fn within(self, root: &Path, package: Manifest) -> Result<Admission, Failure> {
        let mut loaded = self.loaded;
        let workspace = loaded.remove(root).transpose()?;
        let member = Some(package).filter(|candidate| {
            workspace.as_ref().and_then(|workspace| workspace.admits(candidate.directory())).is_some()
        });
        loaded.extend(workspace.map(|workspace| (root.to_path_buf(), Ok(workspace))));
        Ok(Admission { loaded, admitted: self.admitted }.admitting(member))
    }

    fn admitting(self, package: Option<Manifest>) -> Admission {
        Admission { admitted: self.admitted.into_iter().chain(package).collect(), ..self }
    }
}

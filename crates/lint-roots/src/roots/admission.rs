mod workspace;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::regrouped::Regrouped;
use crate::ignored::Ignored;
use file_ports::Files;
use invariants::Ignoring;
use lint_layout::Failure;
use lint_layout::Manifest;
use workspace::Workspace;

#[derive(Debug)]
pub struct Admission;

impl Admission {
    pub fn members_among(
        files: &impl Files,
        groups: Vec<Result<Vec<Manifest>, Failure>>,
    ) -> Vec<Result<Vec<Manifest>, Failure>> {
        let roots: Vec<Vec<PathBuf>> = groups
            .iter()
            .map(|group| group.as_ref().ignoring(Ignored::Unread).map_or_default(|found| Self::roots(found)))
            .collect();
        let loaded = Regrouped::like(&roots, Workspace::load_many(files, &roots.concat()));
        groups
            .into_iter()
            .zip(roots.into_iter().zip(loaded))
            .map(|(group, (group_roots, workspaces))| {
                group.and_then(|found| Self::admitted(found, group_roots.into_iter().zip(workspaces).collect()))
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
        let placed: Vec<Placed> = found
            .into_iter()
            .filter(|manifest| manifest.package().is_some())
            .map(|package| Placed {
                root: Self::enclosing(&workspaces, package.directory()).filter(|_| package.workspace().is_none()),
                package,
            })
            .collect();
        let Loaded { ready, mut failed } = loaded.into_iter().collect();
        let failing = placed.iter().filter_map(|place| place.root.as_ref()).find(|root| failed.contains_key(*root));
        let failure = failing.and_then(|root| failed.remove(root));
        let admitted = placed.into_iter().filter_map(|place| place.admitted(&ready)).collect();
        failure.map_or(Ok(admitted), Err)
    }
}

#[derive(Debug)]
struct Placed {
    root: Option<PathBuf>,
    package: Manifest,
}

impl Placed {
    fn admitted(self, ready: &BTreeMap<PathBuf, Workspace>) -> Option<Manifest> {
        let directory = self.package.directory();
        let enclosing =
            self.root.as_ref().map(|root| ready.get(root).and_then(|workspace| workspace.admits(directory)));
        let verdict = enclosing.map(|admits| admits.map(Path::to_path_buf));
        verdict.as_ref().is_none_or(Option::is_some).then_some(self.package)
    }
}

#[derive(Debug, Default)]
struct Loaded {
    ready: BTreeMap<PathBuf, Workspace>,
    failed: BTreeMap<PathBuf, Failure>,
}

impl FromIterator<(PathBuf, Result<Workspace, Failure>)> for Loaded {
    fn from_iter<Outcomes: IntoIterator<Item = (PathBuf, Result<Workspace, Failure>)>>(outcomes: Outcomes) -> Loaded {
        outcomes.into_iter().map(|(root, outcome)| Loaded::of_outcome(root, outcome)).fold(
            Loaded::default(),
            |mut loaded, part| {
                loaded.ready.extend(part.ready);
                loaded.failed.extend(part.failed);
                loaded
            },
        )
    }
}

impl Loaded {
    fn of_outcome(root: PathBuf, outcome: Result<Workspace, Failure>) -> Loaded {
        match outcome {
            Ok(workspace) => Loaded { ready: BTreeMap::from([(root, workspace)]), failed: BTreeMap::new() },
            Err(failure) => Loaded { ready: BTreeMap::new(), failed: BTreeMap::from([(root, failure)]) },
        }
    }
}

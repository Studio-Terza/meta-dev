mod boundary;

use std::path::{Component, Path, PathBuf};

use file_ports::{Files, Kind};
use grammar::{Construct, Directory, Edit, Edited, Editor, Nested};
use store_ports::Store;

use super::super::Ground;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use crate::{HookEvent, Outcome};
pub(crate) use boundary::Running;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Refusal {
    Unroll(Nested),
    InPlace { editor: Editor, within: Option<Construct> },
    Wrote { path: PathBuf, repository: PathBuf, within: Option<Construct> },
    Moved { path: PathBuf, repository: PathBuf, within: Option<Construct> },
    Unresolved { written: PathBuf, within: Option<Construct> },
}

#[derive(Debug)]
struct Judging<'ground, 'run, F: Files, S: Store> {
    ground: &'ground Ground<'run, F, S>,
    cwd: &'ground Path,
}

impl Running {
    pub(crate) fn answered<F: Files, S: Store>(&self, ground: &Ground<'_, F, S>) -> Result<Uttered, Failure> {
        let command = self.command();
        let judging = Judging { ground, cwd: self.cwd() };
        let refused = Nested::read(command)
            .map(Refusal::Unroll)
            .or_else(|| Edited::read(command).iter().find_map(|edited| judging.refusal(edited)));
        refused
            .map_or_else(|| Ok(Uttered::silent(Ok(Outcome::Hooked(HookEvent::PreToolUse)))), |refusal| refusal.denied())
    }
}

impl<F: Files, S: Store> Judging<'_, '_, F, S> {
    const REPOSITORY: &'static str = ".git";

    fn refusal(&self, edited: &Edited) -> Option<Refusal> {
        let within = edited.construct();
        let directory = edited.directory();
        let unresolved = |written: &Path| Refusal::Unresolved { written: written.to_path_buf(), within };
        match edited.edit() {
            Edit::InPlace(editor) => Some(Refusal::InPlace { editor: *editor, within }),
            Edit::Wrote(written) => match self.resolved(Path::new(written), directory) {
                Some(path) => self.repository(&path).map(|repository| Refusal::Wrote { path, repository, within }),
                None => Some(unresolved(Path::new(written))),
            },
            Edit::Moved { sources, destination } => {
                let blind = sources
                    .iter()
                    .chain([destination])
                    .map(Path::new)
                    .find(|written| self.resolved(written, directory).is_none())
                    .map(unresolved);
                let landing = self.resolved(Path::new(destination), directory).and_then(|path| self.repository(&path));
                blind.or_else(|| {
                    sources.iter().filter_map(|source| self.resolved(Path::new(source), directory)).find_map(|path| {
                        let repository = self.repository(&path).filter(|held| landing.as_ref() != Some(held))?;
                        Some(Refusal::Moved { path, repository, within })
                    })
                })
            }
        }
    }

    fn resolved(&self, written: &Path, directory: &Directory) -> Option<PathBuf> {
        let base = match directory {
            Directory::Started => Some(self.cwd.to_path_buf()),
            Directory::Changed(steps) => Some(steps.iter().fold(self.cwd.to_path_buf(), |base, step| base.join(step))),
            Directory::Unknown => written.is_absolute().then(PathBuf::new),
        }?;
        Some(Self::folded(&base.join(written)))
    }

    fn folded(path: &Path) -> PathBuf {
        path.components().fold(PathBuf::new(), |resolved, component| match component {
            Component::ParentDir => resolved.parent().map(Path::to_path_buf).unwrap_or(resolved),
            Component::CurDir => resolved,
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => resolved.join(component),
        })
    }

    fn repository(&self, path: &Path) -> Option<PathBuf> {
        path.ancestors()
            .find(|directory| self.ground.kind(&directory.join(Self::REPOSITORY)) != Kind::Missing)
            .map(Path::to_path_buf)
    }
}

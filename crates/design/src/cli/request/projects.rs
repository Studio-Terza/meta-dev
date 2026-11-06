mod slug;

use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};

use console_ports::Environment;
use design_store::StreamSchema;
use file_ports::Files;
use local_config::Keeping;
use store_ports::Sqlite;
use transcript_ports::ClaudeCodeDir;

use super::persistence::Persistence;
use crate::cli::failure::Failure;
use slug::Slug;

#[derive(Debug)]
pub struct Projects;

#[derive(Debug)]
pub struct Located {
    transcripts: PathBuf,
    workspace: PathBuf,
    repo: PathBuf,
    persistence: Persistence,
}

impl Located {
    const META: &'static str = ".meta";
    const STORE: &'static str = "design.sqlite";

    pub fn transcripts<F: Files>(&self, files: F) -> ClaudeCodeDir<F> {
        ClaudeCodeDir::new(files, self.transcripts.clone())
    }

    pub fn store(&self, opened: impl FnOnce(PathBuf) -> Sqlite) -> Sqlite {
        match self.persistence {
            Persistence::File => opened(self.workspace.join(Self::META).join(Self::STORE)),
            Persistence::DryRun => Sqlite::in_memory::<StreamSchema>(),
        }
    }

    pub fn keeping(&self) -> Keeping {
        match self.persistence {
            Persistence::File => Keeping::Write,
            Persistence::DryRun => Keeping::Plan,
        }
    }

    pub fn workspace(&self) -> PathBuf {
        self.workspace.clone()
    }

    pub fn repo(&self) -> &Path {
        &self.repo
    }
}

impl Projects {
    pub fn located(
        environment: &impl Environment,
        workspace: Option<&Path>,
        persistence: Persistence,
    ) -> Result<Located, Failure> {
        let home = environment.variable(OsStr::new("HOME")).ok_or(Failure::Unset(OsString::from("HOME")))?;
        let current = environment.variable(OsStr::new("PWD")).map(PathBuf::from);
        let repo = Self::resolved(&current.clone().unwrap_or_default());
        let chosen = Self::chosen(current, workspace).ok_or(Failure::Unset(OsString::from("PWD")))?;
        let root = Self::resolved(&chosen);
        let slug = Slug::try_from(root.as_path())?;
        Ok(Located {
            transcripts: PathBuf::from(home).join(".claude").join("projects").join(PathBuf::from(slug)),
            workspace: root,
            repo,
            persistence,
        })
    }

    fn chosen(current: Option<PathBuf>, workspace: Option<&Path>) -> Option<PathBuf> {
        match workspace {
            Some(workspace) => Some(current.unwrap_or_default().join(workspace)),
            None => current,
        }
    }

    fn resolved(path: &Path) -> PathBuf {
        path.components().fold(PathBuf::new(), Self::stepped)
    }

    fn stepped(resolved: PathBuf, component: Component<'_>) -> PathBuf {
        match component {
            Component::ParentDir => resolved.parent().map(Path::to_path_buf).unwrap_or(resolved),
            Component::CurDir => resolved,
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => resolved.join(component),
        }
    }
}

mod slug;

use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};

use console_ports::Environment;
use file_ports::Files;
use transcript_ports::ClaudeCodeDir;

use crate::cli::failure::Failure;
use slug::Slug;

#[derive(Debug)]
pub struct Projects;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kept {
    Store,
    Agents,
}

#[derive(Debug)]
pub struct Located {
    transcripts: PathBuf,
    workspace: PathBuf,
}

impl Located {
    const META: &'static str = ".meta";
    const STORE: &'static str = "design.sqlite";
    const AGENTS: &'static str = "agents";

    pub fn transcripts<F: Files>(&self, files: F) -> ClaudeCodeDir<F> {
        ClaudeCodeDir::new(files, self.transcripts.clone())
    }

    pub fn kept(&self, kept: Kept) -> PathBuf {
        self.workspace.join(Self::META).join(kept.name())
    }
}

impl Kept {
    fn name(self) -> &'static Path {
        match self {
            Kept::Store => Path::new(Located::STORE),
            Kept::Agents => Path::new(Located::AGENTS),
        }
    }
}

impl Projects {
    pub fn located(environment: &impl Environment, workspace: Option<&Path>) -> Result<Located, Failure> {
        let home = environment.variable(OsStr::new("HOME")).ok_or(Failure::Unset(OsString::from("HOME")))?;
        let current = environment.variable(OsStr::new("PWD")).map(PathBuf::from);
        let chosen = workspace
            .map(|named| current.clone().unwrap_or_default().join(named))
            .or(current)
            .ok_or(Failure::Unset(OsString::from("PWD")))?;
        let root = Self::resolved(&chosen);
        let slug = Slug::try_from(root.as_path())?;
        Ok(Located {
            transcripts: PathBuf::from(home).join(".claude").join("projects").join(PathBuf::from(slug)),
            workspace: root,
        })
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

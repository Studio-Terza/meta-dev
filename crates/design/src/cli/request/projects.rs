mod slug;

use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};

use console_ports::{Environment, Variables};
use file_ports::Files;
use invariants::Ignoring;
use transcript_ports::{ClaudeCodeDir, SessionId};

use super::ignored::Ignored;
use crate::cli::failure::Failure;
use slug::Slug;

#[derive(Debug)]
pub struct Projects;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kept {
    Store,
    Agents,
    Local,
}

#[derive(Clone, Debug)]
pub struct Located {
    home: PathBuf,
    transcripts: PathBuf,
    workspace: PathBuf,
    harness: Option<PathBuf>,
}

impl Located {
    const META: &'static str = ".meta";
    const STORE: &'static str = "design.sqlite";
    const AGENTS: &'static str = "agents";
    const LOCAL: &'static str = "local.toml";
    const CLAUDE: &'static str = ".claude";
    const SETTINGS: &'static str = "settings.local.json";
    const TRANSCRIPT: &'static str = "jsonl";

    pub fn transcripts<F: Files>(&self, files: F) -> ClaudeCodeDir<F> {
        ClaudeCodeDir::new(files, self.transcripts.clone())
    }

    pub fn transcript(&self, session: &SessionId) -> PathBuf {
        self.transcripts.join(format!("{session}.{}", Self::TRANSCRIPT))
    }

    pub fn kept(&self, kept: Kept) -> PathBuf {
        self.workspace.join(kept.within())
    }

    pub fn workspace(&self) -> &Path {
        &self.workspace
    }

    pub fn harness(&self) -> Option<&Path> {
        self.harness.as_deref()
    }

    pub fn settings(&self) -> PathBuf {
        self.workspace.join(Self::CLAUDE).join(Self::SETTINGS)
    }

    pub fn resolved(&self, path: &Path) -> PathBuf {
        self.home.join(path)
    }

    pub fn rendered<'path>(&self, path: &'path Path) -> &'path Path {
        path.strip_prefix(&self.home).ignoring(Ignored::Outside).unwrap_or(path)
    }
}

impl Kept {
    pub fn within(self) -> PathBuf {
        Path::new(Located::META).join(self.name())
    }

    fn name(self) -> &'static Path {
        match self {
            Kept::Store => Path::new(Located::STORE),
            Kept::Agents => Path::new(Located::AGENTS),
            Kept::Local => Path::new(Located::LOCAL),
        }
    }
}

impl Projects {
    pub fn located(
        variables: &Variables,
        workspace: Option<&Path>,
        harness: Option<&Path>,
    ) -> Result<Located, Failure> {
        let home =
            variables.variable(OsStr::new("HOME")).map(PathBuf::from).ok_or(Failure::Unset(OsString::from("HOME")))?;
        let current = variables.variable(OsStr::new("PWD")).map(PathBuf::from);
        let chosen = workspace
            .map(|named| current.clone().unwrap_or_default().join(named))
            .or(current)
            .ok_or(Failure::Unset(OsString::from("PWD")))?;
        let root = Self::resolved(&chosen);
        let slug = Slug::try_from(root.as_path())?;
        Ok(Located {
            transcripts: home.join(Located::CLAUDE).join("projects").join(PathBuf::from(slug)),
            workspace: root,
            harness: harness.map(|named| home.join(named)),
            home,
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

mod slug;

use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};

use console_ports::Environment;
use design_model::Project;
use file_ports::{Files, Kind};
use local_config::LocalConfig;
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
    repo: PathBuf,
    working: PathBuf,
    project: Option<Project>,
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

    pub fn configured(&self, project: Project) -> LocalConfig {
        LocalConfig::new(project, self.workspace.clone())
    }

    pub fn repo(&self) -> &Path {
        &self.repo
    }

    pub fn project(&self) -> Option<&Project> {
        self.project.as_ref()
    }

    pub fn within(&self, path: &Path) -> PathBuf {
        Projects::resolved(&self.working.join(path))
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
    const GIT: &'static str = ".git";

    pub fn located(
        environment: &impl Environment,
        files: &impl Files,
        workspace: Option<&Path>,
    ) -> Result<Located, Failure> {
        let home = environment.variable(OsStr::new("HOME")).ok_or(Failure::Unset(OsString::from("HOME")))?;
        let current = environment.variable(OsStr::new("PWD")).map(PathBuf::from);
        let working = Self::resolved(&current.clone().unwrap_or_default());
        let repo = Self::repository(files, &working);
        let config = LocalConfig::found(files, &repo)?;
        let recorded = config.as_ref().map(|found| found.workspace().to_path_buf());
        let chosen = workspace
            .map(|named| current.clone().unwrap_or_default().join(named))
            .or(recorded)
            .or(current)
            .ok_or(Failure::Unset(OsString::from("PWD")))?;
        let root = Self::resolved(&chosen);
        let slug = Slug::try_from(root.as_path())?;
        Ok(Located {
            transcripts: PathBuf::from(home).join(".claude").join("projects").join(PathBuf::from(slug)),
            workspace: root,
            repo,
            working,
            project: config.map(|found| AsRef::<Project>::as_ref(&found).clone()),
        })
    }

    fn repository(files: &impl Files, here: &Path) -> PathBuf {
        let marks: Vec<PathBuf> = here.ancestors().map(|directory| directory.join(Self::GIT)).collect();
        here.ancestors()
            .zip(files.kinds(&marks))
            .find(|(_directory, kind)| *kind != Kind::Missing)
            .map_or(here, |(directory, _kind)| directory)
            .to_path_buf()
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

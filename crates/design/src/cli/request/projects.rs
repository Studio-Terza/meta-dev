mod slug;

use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};

use console_ports::Environment;
use design_model::Project;
use dispatch_ports::Scratch;
use file_ports::{Files, Kind};
use local_config::LocalConfig;
use transcript_ports::ClaudeCodeDir;

use crate::cli::failure::Failure;
use slug::Slug;

#[derive(Debug)]
pub struct Projects;

#[derive(Debug)]
pub struct Located {
    transcripts: PathBuf,
    workspace: PathBuf,
    repo: PathBuf,
    working: PathBuf,
}

impl Located {
    const META: &'static str = ".meta";
    const STORE: &'static str = "design.sqlite";
    const AGENTS: &'static str = "agents";

    pub fn transcripts<F: Files>(&self, files: F) -> ClaudeCodeDir<F> {
        ClaudeCodeDir::new(files, self.transcripts.clone())
    }

    pub fn store(&self) -> PathBuf {
        self.workspace.join(Self::META).join(Self::STORE)
    }

    pub fn scratch(&self) -> Scratch {
        Scratch::from(self.workspace.join(Self::META).join(Self::AGENTS))
    }

    pub fn configured(&self, project: Project) -> LocalConfig {
        LocalConfig::new(project, self.workspace.clone())
    }

    pub fn repo(&self) -> &Path {
        &self.repo
    }

    pub fn within(&self, path: &Path) -> PathBuf {
        Projects::resolved(&self.working.join(path))
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
        let recorded = LocalConfig::found(files, &repo)?.map(|config| config.workspace().to_path_buf());
        let chosen = workspace
            .map(|workspace| current.clone().unwrap_or_default().join(workspace))
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

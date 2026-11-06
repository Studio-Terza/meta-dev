use std::path::{Path, PathBuf};
use std::str::FromStr;

use design_model::Project;
use file_ports::{Bytes, Files, Kind};
use serde::{Deserialize, Serialize};

use crate::{ConfigFailure, Kept};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LocalConfig {
    project: Project,
    workspace: PathBuf,
}

impl FromStr for LocalConfig {
    type Err = ConfigFailure;

    fn from_str(text: &str) -> Result<LocalConfig, ConfigFailure> {
        Ok(toml::from_str(text)?)
    }
}

impl LocalConfig {
    const META: &'static str = ".meta";
    const FILE: &'static str = "local.toml";
    const IGNORE: &'static str = ".gitignore";
    const IGNORED: &'static [u8] = b"*\n";

    #[must_use]
    pub fn new(project: Project, workspace: PathBuf) -> LocalConfig {
        LocalConfig { project, workspace }
    }

    pub fn kept(&self, files: &impl Files, repo: &Path) -> Result<Kept, ConfigFailure> {
        let meta = repo.join(Self::META);
        Self::found(files, &meta.join(Self::FILE))?
            .map_or_else(|| self.written(files, &meta).map(|_written| Kept::Written), |found| self.compared(found))
    }

    fn found(files: &impl Files, path: &Path) -> Result<Option<LocalConfig>, ConfigFailure> {
        match files.kind(path) {
            Kind::Missing => Ok(None),
            Kind::File | Kind::Directory | Kind::Link => Ok(Some(files.read_text(path)?.as_ref().parse()?)),
        }
    }

    fn compared(&self, found: LocalConfig) -> Result<Kept, ConfigFailure> {
        (found == *self)
            .then_some(Kept::Unchanged)
            .ok_or(ConfigFailure::Elsewhere { project: found.project, workspace: found.workspace })
    }

    fn written(&self, files: &impl Files, meta: &Path) -> Result<PathBuf, ConfigFailure> {
        let path = meta.join(Self::FILE);
        files.create_directories(meta)?;
        files.write(&meta.join(Self::IGNORE), Bytes::new(Self::IGNORED.to_vec()))?;
        files.write(&path, Bytes::new(toml::to_string(self)?.into_bytes()))?;
        Ok(path)
    }
}

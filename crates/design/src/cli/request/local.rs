use std::fmt;
use std::path::{Path, PathBuf};

use grammar::Grammar;
use grammar::repository::RepositoryName;
use invariants::Ignoring;
use serde::Deserialize;

use super::ignored::Ignored;

#[derive(Debug, Default, Deserialize)]
pub struct Local {
    repository: Option<Repository>,
    queue: Option<Rooted>,
    specs: Option<Rooted>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(try_from = "String")]
pub struct Repository {
    owner: String,
    name: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(try_from = "PathBuf")]
pub struct Rooted(PathBuf);

#[derive(Debug, thiserror::Error)]
pub enum Unconfigured {
    #[error("not a repository of the form owner/name")]
    Unrepositoried,
    #[error("not an absolute path")]
    Unrooted,
}

impl Local {
    pub fn written(text: &str) -> Local {
        toml::from_str(text).ignoring(Ignored::Unconfigured).unwrap_or_default()
    }

    pub fn repository(&self) -> Option<&Repository> {
        self.repository.as_ref()
    }

    pub fn queue(&self) -> Option<&Rooted> {
        self.queue.as_ref()
    }

    pub fn specs(&self) -> Option<&Rooted> {
        self.specs.as_ref()
    }
}

impl Repository {
    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl TryFrom<String> for Repository {
    type Error = Unconfigured;

    fn try_from(text: String) -> Result<Repository, Unconfigured> {
        let read = RepositoryName::read(&text).ok_or(Unconfigured::Unrepositoried)?;
        Ok(Repository { owner: read.owner().to_owned(), name: read.name().to_owned() })
    }
}

impl fmt::Display for Repository {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}", self.owner, self.name)
    }
}

impl TryFrom<PathBuf> for Rooted {
    type Error = Unconfigured;

    fn try_from(path: PathBuf) -> Result<Rooted, Unconfigured> {
        path.is_absolute().then_some(Rooted(path)).ok_or(Unconfigured::Unrooted)
    }
}

impl AsRef<Path> for Rooted {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Rooted {
    const SPEC: &'static str = "md";

    pub fn spec(&self, project: &str) -> PathBuf {
        self.0.join(project).with_extension(Rooted::SPEC)
    }
}

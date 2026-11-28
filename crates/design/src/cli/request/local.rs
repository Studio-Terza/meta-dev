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
pub struct Rooted(PathBuf);

#[derive(Debug, thiserror::Error)]
#[error("not a repository of the form owner/name")]
pub struct Unrepositoried;

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
    type Error = Unrepositoried;

    fn try_from(text: String) -> Result<Repository, Unrepositoried> {
        let read = RepositoryName::read(&text).ok_or(Unrepositoried)?;
        Ok(Repository { owner: read.owner().to_owned(), name: read.name().to_owned() })
    }
}

impl fmt::Display for Repository {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}", self.owner, self.name)
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

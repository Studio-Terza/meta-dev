use std::fmt;
use std::path::{Path, PathBuf};

use grammar::RepositoryName;
use invariants::Ignoring;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};

use super::ignored::Ignored;

#[derive(Debug, Default, Deserialize)]
pub struct Local {
    #[serde(default)]
    repository: Keyed<Repository>,
    #[serde(default)]
    queue: Keyed<QueueFile>,
    #[serde(default)]
    specs: Keyed<SpecsFolder>,
}

#[derive(Debug)]
struct Keyed<T>(Option<T>);

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(try_from = "String")]
pub struct Repository {
    owner: String,
    name: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(try_from = "PathBuf")]
pub struct QueueFile(PathBuf);

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(try_from = "PathBuf")]
pub(crate) struct SpecsFolder(PathBuf);

#[derive(Debug, thiserror::Error)]
#[error("not a repository of the form owner/name")]
pub struct Unrepositoried;

#[derive(Debug, thiserror::Error)]
#[error("not an absolute path")]
pub struct Unrooted;

impl Local {
    pub(crate) fn written(text: &str) -> Local {
        toml::from_str(text).ignoring(Ignored::Unconfigured).unwrap_or_default()
    }

    pub fn repository(&self) -> Option<&Repository> {
        self.repository.0.as_ref()
    }

    pub fn queue(&self) -> Option<&QueueFile> {
        self.queue.0.as_ref()
    }

    pub(crate) fn specs(&self) -> Option<&SpecsFolder> {
        self.specs.0.as_ref()
    }
}

impl<T> Default for Keyed<T> {
    fn default() -> Keyed<T> {
        Keyed(None)
    }
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for Keyed<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Keyed<T>, D::Error> {
        let value = toml::Value::deserialize(deserializer)?;
        Ok(Keyed(value.try_into().ignoring(Ignored::Unconfigured)))
    }
}

impl Repository {
    pub(crate) fn owner(&self) -> &str {
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

impl TryFrom<PathBuf> for QueueFile {
    type Error = Unrooted;

    fn try_from(path: PathBuf) -> Result<QueueFile, Unrooted> {
        path.is_absolute().then_some(QueueFile(path)).ok_or(Unrooted)
    }
}

impl TryFrom<PathBuf> for SpecsFolder {
    type Error = Unrooted;

    fn try_from(path: PathBuf) -> Result<SpecsFolder, Unrooted> {
        path.is_absolute().then_some(SpecsFolder(path)).ok_or(Unrooted)
    }
}

impl AsRef<Path> for QueueFile {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl SpecsFolder {
    const SPEC: &'static str = "md";

    pub(crate) fn spec(&self, project: &str) -> PathBuf {
        self.0.join(project).with_extension(SpecsFolder::SPEC)
    }
}

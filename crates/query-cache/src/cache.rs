use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, FileFailure, Files, Text};

use crate::sources::Sources;
use crate::{CacheFile, Describe, Undescribed};

#[derive(Debug)]
pub struct Cache {
    root: PathBuf,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Drift {
    drifted: BTreeSet<CacheFile>,
}

#[derive(Debug)]
struct Compared {
    wanted: BTreeMap<CacheFile, Text>,
    drift: Drift,
}

#[derive(Debug, thiserror::Error)]
pub enum Uncached {
    #[error("cannot read {}: {source}", path.display())]
    Unread { path: PathBuf, source: FileFailure },
    #[error("cannot parse {}: {source}", path.display())]
    Unparsed { path: PathBuf, source: syn::Error },
    #[error(transparent)]
    Undescribed(#[from] Undescribed),
    #[error("cannot write a cached query: {0}")]
    Unwritten(#[from] serde_json::Error),
    #[error("cannot rewrite {}: {source}", path.display())]
    Unrewritten { path: PathBuf, source: FileFailure },
}

impl Cache {
    const DIRECTORY: &'static str = ".sqlx";
    const CRATES: &'static str = "crates";

    #[must_use]
    pub fn rooted(root: &Path) -> Cache {
        Cache { root: root.to_path_buf() }
    }

    pub fn drifted(&self, files: &impl Files, described: &impl Describe) -> Result<Drift, Uncached> {
        self.compared(files, described).map(|compared| compared.drift)
    }

    pub fn recorded(&self, files: &impl Files, described: &impl Describe) -> Result<Drift, Uncached> {
        let Compared { wanted, drift } = self.compared(files, described)?;
        drift.drifted.iter().try_for_each(|name| {
            let path = self.directory().join(name.as_ref());
            let rewritten =
                wanted.get(name).map_or_else(|| files.remove(&path), |text| files.write(&path, Bytes::from(text)));
            rewritten.map_err(|source| Uncached::Unrewritten { path, source })
        })?;
        Ok(drift)
    }

    fn directory(&self) -> PathBuf {
        self.root.join(Cache::DIRECTORY)
    }

    fn compared(&self, files: &impl Files, described: &impl Describe) -> Result<Compared, Uncached> {
        let wanted = Sources::queries(files, &self.root.join(Cache::CRATES))?
            .iter()
            .map(|query| {
                let entry = described.describe(&query.value())?;
                Ok((entry.file(), Text::new(entry.written()?)))
            })
            .collect::<Result<BTreeMap<CacheFile, Text>, Uncached>>()?;
        let directory = self.directory();
        let held = files
            .entries(&directory)
            .map_err(|source| Uncached::Unread { path: directory.clone(), source })?
            .into_iter()
            .filter_map(|path| {
                path.file_name().map(|name| (CacheFile::from(name.to_string_lossy().into_owned()), path.clone()))
            })
            .map(|(name, path)| {
                let text = files.read_text(&path).map_err(|source| Uncached::Unread { path, source })?;
                Ok((name, text))
            })
            .collect::<Result<BTreeMap<CacheFile, Text>, Uncached>>()?;
        let drifted =
            wanted.keys().chain(held.keys()).filter(|name| wanted.get(*name) != held.get(*name)).cloned().collect();
        Ok(Compared { wanted, drift: Drift { drifted } })
    }
}

impl Drift {
    #[must_use]
    pub fn settled(&self) -> Option<()> {
        self.drifted.first().map_or(Some(()), |_drifted| None)
    }
}

impl fmt::Display for Drift {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.drifted.iter().try_for_each(|name| writeln!(formatter, "{name}"))
    }
}

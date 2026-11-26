use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, FileFailure, Files};
use lint_core::{FileHead, SourceFile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{Filings, Worked};
use crate::cli::ignored::Ignored;
use crate::cli::timings::Timings;
use invariants::Ignoring;

#[derive(Debug)]
pub(crate) struct Invocation {
    shown: Timings,
    binary: PathBuf,
    store: PathBuf,
}

impl Invocation {
    const STORE: [&'static str; 3] = ["target", "lint-cache", "files"];

    pub(crate) fn new(shown: Timings, binary: PathBuf, roots: &[PathBuf]) -> Invocation {
        let root = roots.first().map_or_else(PathBuf::new, Clone::clone);
        Invocation { shown, binary, store: Self::STORE.iter().fold(root, |path, part| path.join(part)) }
    }

    pub(crate) fn shown(&self) -> Timings {
        self.shown
    }

    pub(crate) fn recalled(&self, files: &impl Files) -> Store {
        let binary = files.read(&self.binary).ignoring(Ignored::Unhashed).map(|bytes| Binary::from(&bytes));
        let held = binary.and_then(|_| files.read(&self.store).ignoring(Ignored::Unremembered));
        let kept =
            held.and_then(|bytes| serde_json::from_slice::<Vec<Kept>>(bytes.as_ref()).ignoring(Ignored::Unrecalled));
        let filings = kept.map_or_default(|entries| {
            entries.into_iter().fold(BTreeMap::new(), |recalled, entry| entry.recalled(recalled))
        });
        Store { binary, filings, path: self.store.clone() }
    }
}

#[derive(Clone, Copy, Debug)]
struct Binary([u8; Binary::DIGEST]);

impl Binary {
    const DIGEST: usize = 32;

    fn key(self, file: &SourceFile) -> Key {
        Key(Self::fed(Sha256::new().chain_update(self.0), file).finalize().into())
    }

    fn fed(hasher: Sha256, file: &SourceFile) -> Sha256 {
        let described = Described(file.head()).to_string();
        let modules = file.test_modules();
        let framed = Self::framed(Self::framed(hasher, described.as_bytes()), file.text().as_ref().as_bytes());
        modules.iter().fold(framed.chain_update(modules.len().to_le_bytes()), Self::fed)
    }

    fn framed(hasher: Sha256, bytes: &[u8]) -> Sha256 {
        hasher.chain_update(bytes.len().to_le_bytes()).chain_update(bytes)
    }
}

impl From<&Bytes> for Binary {
    fn from(bytes: &Bytes) -> Binary {
        Binary(Sha256::digest(bytes.as_ref()).into())
    }
}

#[derive(Clone, Copy, Debug)]
struct Described<'head>(&'head FileHead);

impl fmt::Display for Described<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let head = self.0;
        let path: &Path = head.as_ref();
        write!(formatter, "{}{:?}{:?}{:?}", path.display(), head.located(), head.placement(), head.crate_file())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct Key([u8; Binary::DIGEST]);

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Kept {
    key: Key,
    filings: Filings,
}

impl Kept {
    pub(crate) fn remembered(key: Key, worked: &Worked) -> Kept {
        Kept { key, filings: worked.remembered() }
    }

    fn recalled(self, mut recalled: BTreeMap<Key, Filings>) -> BTreeMap<Key, Filings> {
        recalled.insert(self.key, self.filings);
        recalled
    }
}

#[derive(Debug)]
pub(crate) struct Store {
    binary: Option<Binary>,
    filings: BTreeMap<Key, Filings>,
    path: PathBuf,
}

impl Store {
    pub(crate) fn key(&self, file: &SourceFile) -> Option<Key> {
        self.binary.map(|binary| binary.key(file))
    }

    pub(crate) fn recalled(&self, key: &Key) -> Option<Filings> {
        self.filings.get(key).cloned()
    }

    pub(crate) fn written(&self, kept: &[Kept], files: &impl Files) -> Result<(), FileFailure> {
        self.path.parent().map_or(Ok(()), |directory| files.create_directories(directory))?;
        let encoded = serde_json::to_vec(kept).ignoring(Ignored::Unremembered);
        encoded.map_or(Ok(()), |bytes| files.write(&self.path, Bytes::new(bytes)))
    }
}

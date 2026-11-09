use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use newtype::Wrapping;

mod boundary;

use boundary::Recall;

use crate::{Bytes, FileFailure, Files, Kind, Size, Text};

#[derive(Debug)]
pub struct Cached<'files, F> {
    files: &'files F,
    reads: Memo<Bytes>,
    texts: Memo<Text>,
    listings: Memo<Vec<PathBuf>>,
    canonicals: Memo<PathBuf>,
    kinds: Memo<Kind>,
}

#[derive(Debug)]
struct Memo<Value>(Mutex<BTreeMap<PathBuf, Arc<OnceLock<Recall<Value>>>>>);

impl<Value: Clone> Memo<Value> {
    fn empty() -> Memo<Value> {
        Memo(Mutex::new(BTreeMap::new()))
    }

    fn recalled(&self, path: &Path, fresh: impl Fn() -> Result<Value, FileFailure>) -> Result<Value, FileFailure> {
        self.slot(path).get_or_init(|| Recall::remembered(fresh())).replayed(fresh)
    }

    fn slot(&self, path: &Path) -> Arc<OnceLock<Recall<Value>>> {
        let mut slots = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let found = slots.get(path).map(Arc::clone);
        found.unwrap_or_else(|| Arc::clone(slots.entry(path.to_path_buf()).or_default()))
    }

    fn forgotten(&self) {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).clear();
    }
}

impl<'files, F: Files> Cached<'files, F> {
    #[must_use]
    pub fn new(files: &'files F) -> Cached<'files, F> {
        Cached {
            files,
            reads: Memo::empty(),
            texts: Memo::empty(),
            listings: Memo::empty(),
            canonicals: Memo::empty(),
            kinds: Memo::empty(),
        }
    }

    fn changed<Done>(&self, done: Done) -> Done {
        self.reads.forgotten();
        self.texts.forgotten();
        self.listings.forgotten();
        self.canonicals.forgotten();
        self.kinds.forgotten();
        done
    }

    fn decoded(&self, path: &Path) -> Result<Text, FileFailure> {
        let read = self.read(path).ok().and_then(|bytes| String::from_utf8(bytes.into_inner()).ok());
        read.map(Text::new).map_or_else(|| self.files.read_text(path), Ok)
    }
}

impl<F: Files> Files for Cached<'_, F> {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure> {
        self.reads.recalled(path, || self.files.read(path))
    }

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure> {
        self.texts.recalled(path, || self.decoded(path))
    }

    fn write(&self, path: &Path, bytes: Bytes) -> Result<(), FileFailure> {
        self.changed(self.files.write(path, bytes))
    }

    fn append(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure> {
        self.changed(self.files.append(path, bytes))
    }

    fn create_directories(&self, path: &Path) -> Result<(), FileFailure> {
        self.changed(self.files.create_directories(path))
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FileFailure> {
        self.changed(self.files.copy(from, to))
    }

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure> {
        self.listings.recalled(directory, || self.files.entries(directory))
    }

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure> {
        self.canonicals.recalled(path, || self.files.canonical(path))
    }

    fn kind(&self, path: &Path) -> Kind {
        self.kinds.recalled(path, || Ok(self.files.kind(path))).unwrap_or_else(|_| self.files.kind(path))
    }

    fn size(&self, path: &Path) -> Result<Size, FileFailure> {
        self.files.size(path)
    }
}

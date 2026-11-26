use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use invariants::{Ignoring, Unpoisoned};
use newtype::Wrapping;

mod boundary;

use boundary::Recall;

use crate::{Bytes, FileFailure, Files, Ignored, Kind, Size, Text};

#[derive(Debug)]
pub struct Cached<'files, F> {
    files: &'files F,
    reads: Memo<Bytes>,
    texts: Memo<Text>,
    listings: Memo<Vec<PathBuf>>,
    canonicals: Memo<PathBuf>,
    kinds: Memo<Kind>,
}

type Slot<Value> = Arc<OnceLock<Recall<Value>>>;

#[derive(Debug)]
struct Memo<Value>(Mutex<HashMap<PathBuf, Slot<Value>>>);

impl<Value: Clone> Memo<Value> {
    fn recalled(&self, path: &Path, fresh: impl Fn() -> Result<Value, FileFailure>) -> Result<Value, FileFailure> {
        self.slot(path).get_or_init(|| Recall::remembered(fresh())).replayed(fresh)
    }

    fn recalled_many(
        &self,
        paths: &[impl AsRef<Path>],
        fresh: impl Fn(&Path) -> Result<Value, FileFailure>,
        batch: impl FnOnce(&[&Path]) -> Vec<Result<Value, FileFailure>>,
    ) -> Vec<Result<Value, FileFailure>> {
        let held: Vec<&Path> = paths.iter().map(AsRef::as_ref).collect();
        let slots: Vec<Slot<Value>> = held.iter().map(|path| self.slot(path)).collect();
        let (missing, open): (Vec<&Path>, Vec<&Slot<Value>>) =
            held.iter().copied().zip(&slots).filter(|(_, slot)| slot.get().is_none()).unzip();
        open.into_iter().zip(batch(&missing)).for_each(|(slot, found)| {
            slot.get_or_init(|| Recall::remembered(found));
        });
        held.into_iter()
            .zip(slots)
            .map(|(path, slot)| slot.get_or_init(|| Recall::remembered(fresh(path))).replayed(|| fresh(path)))
            .collect()
    }

    fn slot(&self, path: &Path) -> Slot<Value> {
        let mut slots = self.0.lock().unpoisoned();
        let found = slots.get(path).map(Arc::clone);
        found.unwrap_or_else(|| Arc::clone(slots.entry(path.to_path_buf()).or_default()))
    }

    fn forgotten(&self) {
        self.0.lock().unpoisoned().clear();
    }
}

impl<'files, F: Files> Cached<'files, F> {
    #[must_use]
    pub fn new(files: &'files F) -> Cached<'files, F> {
        Cached {
            files,
            reads: Memo(Mutex::default()),
            texts: Memo(Mutex::default()),
            listings: Memo(Mutex::default()),
            canonicals: Memo(Mutex::default()),
            kinds: Memo(Mutex::default()),
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
        let read = self
            .read(path)
            .ignoring(Ignored::Undecoded)
            .and_then(|bytes| String::from_utf8(bytes.into_inner()).ignoring(Ignored::Undecoded));
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
        self.kinds
            .recalled(path, || Ok(self.files.kind(path)))
            .ignoring(Ignored::KindRecalled)
            .unwrap_or_else(|| self.files.kind(path))
    }

    fn size(&self, path: &Path) -> Result<Size, FileFailure> {
        self.files.size(path)
    }

    fn read_many(&self, paths: &[impl AsRef<Path>]) -> Vec<Result<Bytes, FileFailure>> {
        self.reads.recalled_many(paths, |path| self.files.read(path), |missing| self.files.read_many(missing))
    }

    fn entries_many(&self, directories: &[impl AsRef<Path>]) -> Vec<Result<Vec<PathBuf>, FileFailure>> {
        self.listings.recalled_many(
            directories,
            |directory| self.files.entries(directory),
            |missing| self.files.entries_many(missing),
        )
    }

    fn canonical_many(&self, paths: &[impl AsRef<Path>]) -> Vec<Result<PathBuf, FileFailure>> {
        self.canonicals.recalled_many(
            paths,
            |path| self.files.canonical(path),
            |missing| self.files.canonical_many(missing),
        )
    }

    fn kinds(&self, paths: &[impl AsRef<Path>]) -> Vec<Kind> {
        let recalled = self.kinds.recalled_many(
            paths,
            |path| Ok(self.files.kind(path)),
            |missing| self.files.kinds(missing).into_iter().map(Ok).collect(),
        );
        paths
            .iter()
            .zip(recalled)
            .map(|(path, kind)| kind.ignoring(Ignored::KindRecalled).unwrap_or_else(|| self.files.kind(path.as_ref())))
            .collect()
    }
}

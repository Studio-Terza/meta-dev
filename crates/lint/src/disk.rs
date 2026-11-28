use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use file_ports::{Files, Kind};
use invariants::Ignoring;
use syntax::{Clock, Failure, Tree};
use timer_ports::{Moment, Timer};

use crate::ignored::Ignored;

#[derive(Debug)]
pub(crate) struct Disked<'files, F>(&'files F);

#[derive(Debug)]
pub(crate) struct Timed<'timer, T> {
    timer: &'timer T,
    origin: Moment,
}

impl<'files, F: Files> Disked<'files, F> {
    const HIDDEN: u8 = b'.';

    pub(crate) fn new(files: &'files F) -> Disked<'files, F> {
        Disked(files)
    }

    pub(crate) fn tree(&self, working: &Path, given: &Path) -> Result<Tree, Failure> {
        let files = self.0;
        let canonical = files
            .canonical(&working.join(given))
            .map_err(|source| Failure::Unread { path: given.to_path_buf(), source: source.into() })?;
        let above: BTreeMap<PathBuf, String> = canonical
            .ancestors()
            .skip(1)
            .filter_map(|directory| {
                let manifest = directory.join(Tree::MANIFEST);
                files.read_text(&manifest).ignoring(Ignored::Unlisted).map(|text| (manifest, text.as_ref().to_owned()))
            })
            .collect();
        let outline = Tree::new(given.to_path_buf(), canonical, above);
        let below = self.listed(outline.root()).into_iter().map(|path| match files.read_text(&path) {
            Ok(text) => Ok((path, text.as_ref().to_owned())),
            Err(source) => Err(Failure::Unread { path: outline.shown(&path).to_path_buf(), source: source.into() }),
        });
        let read = below.collect::<Result<BTreeMap<PathBuf, String>, Failure>>()?;
        Ok(outline.with(read))
    }

    fn listed(&self, directory: &Path) -> Vec<PathBuf> {
        let files = self.0;
        let manifested = files.kind(&directory.join(Tree::MANIFEST));
        let entries = files.entries(directory).ignoring(Ignored::Unlisted).unwrap_or_default();
        entries
            .into_iter()
            .flat_map(|entry| {
                let name = entry.file_name().unwrap_or_default();
                let hidden = name.as_encoded_bytes().first() == Some(&Disked::<F>::HIDDEN);
                let built = manifested == Kind::File && name == Tree::BUILD_OUTPUT;
                let source = name == Tree::MANIFEST || entry.extension() == Some(OsStr::new(Tree::SOURCE_EXTENSION));
                match files.kind(&entry) {
                    Kind::Directory if !hidden && !built => self.listed(&entry),
                    Kind::File if source => vec![entry],
                    Kind::Directory | Kind::File | Kind::Link | Kind::Missing => Vec::new(),
                }
            })
            .collect()
    }
}

impl<'timer, T: Timer> Timed<'timer, T> {
    pub(crate) fn new(timer: &'timer T) -> Timed<'timer, T> {
        Timed { timer, origin: timer.now() }
    }
}

impl<T: Timer> Clock for Timed<'_, T> {
    fn now(&self) -> Duration {
        self.timer.now().since(self.origin).into()
    }
}

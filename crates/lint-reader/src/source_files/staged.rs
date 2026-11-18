mod repo_files;

use std::io;
use std::path::Path;
use std::sync::Mutex;
use std::thread::{self, ScopedJoinHandle};

use file_ports::Files;
use lint_core::RepoFile;
use lint_layout::Failure;
use lint_roots::{Below, Listing, Roots, Target, Tree};

use super::located::{Located, Node};
use invariants::{Rejoined, Unpoisoned};
use repo_files::RepoFiles;

type Listed = Vec<Result<Vec<RepoFile>, Failure>>;

#[derive(Debug)]
pub struct Staged {
    targets: Vec<Target>,
    repo_files: Vec<RepoFile>,
}

impl Staged {
    pub fn under(files: &(impl Files + Sync), trees: &[&Tree<'_>]) -> Vec<Result<Staged, Failure>> {
        Roots::surveyed(files, trees)
            .parted(|below, listing| Self::gathered(files, trees, below, &Mutex::new(Some(listing))))
    }

    fn gathered(
        files: &(impl Files + Sync),
        trees: &[&Tree<'_>],
        below: Below,
        parked: &Mutex<Option<Listing>>,
    ) -> Vec<Result<Staged, Failure>> {
        let (targets, repo_files) = thread::scope(|scope| {
            let reading = thread::Builder::new().spawn_scoped(scope, || Self::read(files, trees, parked));
            (Roots::under(files, trees, below), Self::listed(reading, files, trees, parked))
        });
        trees
            .iter()
            .zip(targets)
            .zip(repo_files)
            .map(|((tree, rooted), listed)| {
                let found = rooted?;
                found.first().ok_or_else(|| tree.without_sources())?;
                Ok(Staged { targets: found, repo_files: listed? })
            })
            .collect()
    }

    fn listed(
        reading: io::Result<ScopedJoinHandle<'_, Listed>>,
        files: &impl Files,
        trees: &[&Tree<'_>],
        parked: &Mutex<Option<Listing>>,
    ) -> Listed {
        match reading {
            Ok(handle) => handle.join().rejoined(),
            Err(_) => Self::read(files, trees, parked),
        }
    }

    fn read(files: &impl Files, trees: &[&Tree<'_>], parked: &Mutex<Option<Listing>>) -> Listed {
        let taken = parked.lock().unpoisoned().take();
        taken.map_or_default(|listing| RepoFiles::under(files, trees, listing))
    }

    #[must_use]
    pub fn starts<'tree>(&self, tree: &'tree Tree<'tree>) -> Vec<Node<'tree>> {
        self.targets
            .iter()
            .flat_map(|target| {
                target.files().iter().map(move |file| {
                    let children = file.parent().map_or_default(Path::to_path_buf);
                    Located::new(tree, target.package(), file).node(&children)
                })
            })
            .collect()
    }

    #[must_use]
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }

    #[must_use]
    pub fn repo_files(&self) -> &[RepoFile] {
        &self.repo_files
    }
}

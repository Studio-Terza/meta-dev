mod ignored;
mod kept;

use std::iter;
use std::path::{Path, PathBuf};

use file_ports::{Files, Kind};
use lint_layout::Failure;
use newtype::{Arbitrary, Newtype, Wrapping};

use crate::survey::{Fact, Failed, Survey};
use ignored::Ignored;
use kept::Kept;

#[derive(Newtype, Arbitrary, Clone, Debug)]
pub struct Directory(PathBuf);

#[derive(Debug)]
pub struct Below(Vec<Result<Vec<Directory>, Failure>>);

impl Below {
    #[must_use]
    pub fn trees(self) -> Vec<Result<Vec<Directory>, Failure>> {
        self.0
    }
}

#[derive(Debug)]
struct Walk<'root> {
    root: &'root Path,
    below: Vec<Directory>,
}

impl Directory {
    pub fn trees(files: &impl Files, roots: &[PathBuf]) -> Vec<Result<Vec<Directory>, Failure>> {
        Self::surveyed(files, roots, &Ignored::FOR_PACKAGES).settling(|known, failed| Walk::trees(known, failed, roots))
    }

    pub fn below(files: &impl Files, roots: &[PathBuf]) -> Below {
        Below(Self::trees(files, roots))
    }

    fn surveyed(files: &impl Files, level: &[PathBuf], ignored: &[Ignored]) -> Survey {
        let listed = Survey::learned(files, Fact::Listed, level);
        let entries: Vec<PathBuf> = level.iter().flat_map(|directory| listed.entries(directory)).cloned().collect();
        let with_kinds = listed.combine(Survey::learned(files, Fact::Kind, &entries));
        let manifests: Vec<PathBuf> = Ignored::manifests(level)
            .into_iter()
            .filter(|manifest| with_kinds.kind(manifest) != Kind::Missing)
            .collect();
        let survey = with_kinds.combine(Survey::learned(files, Fact::Followed, &manifests));
        let kept = Kept::new(&survey, ignored);
        let next: Vec<Directory> = level.iter().flat_map(|directory| kept.below(directory)).collect();
        survey.combine(Self::descended(files, next, ignored))
    }

    fn descended(files: &impl Files, next: Vec<Directory>, ignored: &[Ignored]) -> Survey {
        match next.first() {
            Some(_) => {
                Self::surveyed(files, &next.into_iter().map(Wrapping::into_inner).collect::<Vec<PathBuf>>(), ignored)
            }
            None => Survey::default(),
        }
    }
}

impl<'root> Walk<'root> {
    fn all(known: &Survey, roots: &'root [PathBuf], ignored: &[Ignored]) -> Vec<Walk<'root>> {
        let kept = Kept::new(known, ignored);
        roots.iter().map(|root| Walk { root, below: kept.preorder(kept.below(root)) }).collect()
    }

    fn order(&self) -> Vec<&Path> {
        iter::once(self.root).chain(self.below.iter().map(AsRef::as_ref)).collect()
    }

    fn trees(known: &Survey, failed: Failed, roots: &'root [PathBuf]) -> Vec<Result<Vec<Directory>, Failure>> {
        let walks = Walk::all(known, roots, &Ignored::FOR_PACKAGES);
        let orders: Vec<Vec<&Path>> = walks.iter().map(Walk::order).collect();
        let outcomes = failed.settled(known, &orders);
        outcomes.into_iter().zip(walks).map(|(outcome, walk)| outcome.map(|_| walk.below)).collect()
    }
}

impl AsRef<Path> for Directory {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

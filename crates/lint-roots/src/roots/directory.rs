mod gathered;
mod ignored;
mod kept;

use std::iter;
use std::path::{Path, PathBuf};

use file_ports::{Files, Kind};
use lint_layout::Failure;
use newtype::{Arbitrary, Newtype, Wrapping};

use crate::survey::{Fact, Survey};
use gathered::Gathered;
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
pub struct Listing(Vec<Result<Vec<PathBuf>, Failure>>);

impl Listing {
    #[must_use]
    pub fn files(self) -> Vec<Result<Vec<PathBuf>, Failure>> {
        self.0
    }
}

#[derive(Debug)]
pub struct Surveyed {
    below: Below,
    listing: Listing,
}

impl Surveyed {
    pub fn parted<Made>(self, part: impl FnOnce(Below, Listing) -> Made) -> Made {
        part(self.below, self.listing)
    }
}

impl Directory {
    pub fn trees(files: &impl Files, roots: &[PathBuf]) -> Vec<Result<Vec<Directory>, Failure>> {
        Self::walked(files, roots, &Ignored::FOR_PACKAGES, |_, _, order| order)
    }

    pub fn shared(files: &impl Files, roots: &[PathBuf]) -> Surveyed {
        let mut survey = Self::surveyed(files, roots, &Ignored::FOR_FILES, Survey::default());
        let below = Self::found(&mut survey, roots, &Ignored::FOR_PACKAGES, |_, _, order| order);
        let listing = Self::found(&mut survey, roots, &Ignored::FOR_FILES, |kept, root, _| kept.files(root));
        Surveyed { below: Below(below), listing: Listing(listing) }
    }

    fn walked<Found>(
        files: &impl Files,
        roots: &[PathBuf],
        ignored: &[Ignored],
        found: impl Fn(&Gathered<'_>, &Path, Vec<Directory>) -> Found,
    ) -> Vec<Result<Found, Failure>> {
        let mut survey = Self::surveyed(files, roots, ignored, Survey::default());
        Self::found(&mut survey, roots, ignored, found)
    }

    fn found<Found>(
        survey: &mut Survey,
        roots: &[PathBuf],
        ignored: &[Ignored],
        found: impl Fn(&Gathered<'_>, &Path, Vec<Directory>) -> Found,
    ) -> Vec<Result<Found, Failure>> {
        roots
            .iter()
            .map(|root| {
                let kept = Kept::new(survey, ignored);
                let below = kept.preorder(kept.below(root));
                let order: Vec<&Path> = iter::once(root.as_path()).chain(below.iter().map(AsRef::as_ref)).collect();
                survey.settled(&order).map(|settled| found(&Gathered::new(settled, ignored), root, below))
            })
            .collect()
    }

    fn surveyed(files: &impl Files, level: &[PathBuf], ignored: &[Ignored], known: Survey) -> Survey {
        let listed = known.learn(files, Fact::Listed, level);
        let entries: Vec<PathBuf> = level.iter().flat_map(|directory| listed.entries(directory)).cloned().collect();
        let with_kinds = listed.learn(files, Fact::Kind, &entries);
        let manifests: Vec<PathBuf> = Ignored::manifests(level)
            .into_iter()
            .filter(|manifest| with_kinds.kind(manifest) != Kind::Missing)
            .collect();
        let survey = with_kinds.learn(files, Fact::Followed, &manifests);
        let kept = Kept::new(&survey, ignored);
        let next: Vec<Directory> = level.iter().flat_map(|directory| kept.below(directory)).collect();
        Self::descended(files, next, ignored, survey)
    }

    fn descended(files: &impl Files, next: Vec<Directory>, ignored: &[Ignored], survey: Survey) -> Survey {
        match next.first() {
            Some(_) => Self::surveyed(
                files,
                &next.into_iter().map(Wrapping::into_inner).collect::<Vec<PathBuf>>(),
                ignored,
                survey,
            ),
            None => survey,
        }
    }
}

impl AsRef<Path> for Directory {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

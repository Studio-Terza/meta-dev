mod ignored;
mod kept;

use std::path::{Path, PathBuf};

use file_ports::Files;
use lint_layout::Failure;
use newtype::{Arbitrary, Newtype};

use crate::survey::{Fact, Survey};
use ignored::Ignored;
use kept::Kept;

#[derive(Newtype, Arbitrary, Clone, Debug)]
pub struct Directory(PathBuf);

impl Directory {
    pub fn trees(files: &impl Files, roots: &[PathBuf]) -> Vec<Result<Vec<Directory>, Failure>> {
        Self::walked(files, roots, &Ignored::FOR_PACKAGES, |_, _, order| order)
    }

    pub fn files(files: &impl Files, roots: &[PathBuf]) -> Vec<Result<Vec<PathBuf>, Failure>> {
        Self::walked(files, roots, &Ignored::FOR_FILES, |kept, root, _| kept.files(root))
    }

    fn walked<Found>(
        files: &impl Files,
        roots: &[PathBuf],
        ignored: &[Ignored],
        found: impl Fn(&Kept<'_>, &Path, Vec<Directory>) -> Found,
    ) -> Vec<Result<Found, Failure>> {
        let mut survey = Self::surveyed(files, roots, ignored, Survey::default());
        roots
            .iter()
            .map(|root| {
                let order = Kept::new(&survey, ignored).preorder(Directory(root.clone()));
                survey.failure(&order).map_or_else(|| Ok(found(&Kept::new(&survey, ignored), root, order)), Err)
            })
            .collect()
    }

    fn surveyed(files: &impl Files, level: &[PathBuf], ignored: &[Ignored], survey: Survey) -> Survey {
        let survey = survey.learn(files, Fact::Listed, level);
        let entries: Vec<PathBuf> = level.iter().flat_map(|directory| survey.entries(directory)).cloned().collect();
        let survey = survey.learn(files, Fact::Kind, &entries);
        let manifests = Ignored::manifests(level);
        let survey = survey.learn(files, Fact::Followed, &manifests);
        let kept = Kept::new(&survey, ignored);
        let next: Vec<PathBuf> =
            level.iter().flat_map(|directory| kept.below(directory)).map(Newtype::into_inner).collect();
        if next.is_empty() { survey } else { Self::surveyed(files, &next, ignored, survey) }
    }
}

impl AsRef<Path> for Directory {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

use std::path::{Path, PathBuf};

use grammar::slug::Letters;
use newtype::{Arbitrary, Newtype, Wrapping};

use crate::cli::failure::Failure;

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
pub struct Slug(String);

impl TryFrom<&Path> for Slug {
    type Error = Failure;

    fn try_from(workspace: &Path) -> Result<Slug, Failure> {
        workspace
            .to_str()
            .ok_or_else(|| Failure::Unreadable(workspace.to_path_buf()))
            .map(Letters::slugged)
            .map(Slug::new)
    }
}

impl From<Slug> for PathBuf {
    fn from(slug: Slug) -> PathBuf {
        PathBuf::from(slug.into_inner())
    }
}

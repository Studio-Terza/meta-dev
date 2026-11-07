mod ancestor;
mod index;
mod inside_sibling;
mod target;

pub use ancestor::Ancestor;
pub use index::ModuleIndex;
pub use inside_sibling::InsideSibling;
pub use target::Target;

use std::fmt;
use std::path::Path;

use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateRoot;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModulePath<'file> {
    crate_root: &'file CrateRoot,
    segments: &'file [Name],
    file: &'file Path,
}

impl<'file> ModulePath<'file> {
    pub fn of_file(file: &'file SourceFile) -> Self {
        ModulePath {
            crate_root: file.placement().crate_root(),
            segments: file.placement().module().segments(),
            file: file.path(),
        }
    }

    pub const fn file(&self) -> &'file Path {
        self.file
    }

    pub fn parent(&self) -> Option<Self> {
        let (_, parents) = self.segments.split_last()?;
        Some(self.with_segments(parents))
    }

    pub fn truncated_like(&self, other: &Self) -> Self {
        self.with_segments(self.segments.get(..other.segments.len()).unwrap_or(self.segments))
    }

    fn with_segments(&self, segments: &'file [Name]) -> Self {
        ModulePath { crate_root: self.crate_root, segments, file: Path::new("") }
    }
}

impl fmt::Display for ModulePath<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate")?;
        self.segments.iter().try_for_each(|segment| write!(formatter, "::{segment}"))
    }
}

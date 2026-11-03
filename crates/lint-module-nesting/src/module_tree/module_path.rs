mod ancestor;
mod index;
mod inside_sibling;
mod relation;
mod target;

pub use ancestor::Ancestor;
pub use index::ModuleIndex;
pub use inside_sibling::InsideSibling;
pub use relation::Relation;
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
    const MODULE_FILE: &'static str = "mod.rs";

    pub fn of_file(file: &'file SourceFile) -> ModulePath<'file> {
        ModulePath {
            crate_root: file.placement().crate_root(),
            segments: file.placement().module().segments(),
            file: file.path(),
        }
    }

    pub fn file(&self) -> &'file Path {
        self.file
    }

    pub fn parent(&self) -> Option<ModulePath<'file>> {
        let (_, parents) = self.segments.split_last()?;
        Some(self.with_segments(parents))
    }

    pub fn relates<'other, R: Relation>(&self, other: &'other ModulePath<'file>) -> Option<&'other ModulePath<'file>> {
        R::holds(self, other)
    }

    pub fn truncated_like(&self, other: &ModulePath<'file>) -> ModulePath<'file> {
        self.with_segments(self.segments.get(..other.segments.len()).unwrap_or(self.segments))
    }

    fn with_segments(&self, segments: &'file [Name]) -> ModulePath<'file> {
        ModulePath { crate_root: self.crate_root, segments, file: Path::new("") }
    }

    fn directory_module(&self) -> Option<&ModulePath<'file>> {
        self.file.file_name().filter(|name| *name == Self::MODULE_FILE).map(|_| self)
    }
}

impl fmt::Display for ModulePath<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate")?;
        self.segments.iter().try_for_each(|segment| write!(formatter, "::{segment}"))
    }
}

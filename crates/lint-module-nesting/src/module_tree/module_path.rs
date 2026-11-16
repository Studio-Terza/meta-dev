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

use lint_layout::Name;
use lint_placement::{CrateRoot, Nest};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModulePath<'file> {
    nest: Nest<'file, &'file CrateRoot>,
    file: &'file Path,
}

impl<'file> ModulePath<'file> {
    pub fn new(crate_root: &'file CrateRoot, segments: &'file [Name], file: &'file Path) -> ModulePath<'file> {
        ModulePath { nest: Nest::module(crate_root, segments), file }
    }

    pub fn file(&self) -> &'file Path {
        self.file
    }

    pub fn truncated_like(&self, other: &ModulePath<'file>) -> ModulePath<'file> {
        let segments = self.segments();
        let kept = segments.get(..other.segments().len()).unwrap_or(segments);
        ModulePath { nest: self.nest.krate().map_or(self.nest, |root| Nest::module(root, kept)), file: Path::new("") }
    }

    fn segments(&self) -> &'file [Name] {
        <&[Name]>::from(self.nest)
    }
}

impl<'file> AsRef<Nest<'file, &'file CrateRoot>> for ModulePath<'file> {
    fn as_ref(&self) -> &Nest<'file, &'file CrateRoot> {
        &self.nest
    }
}

impl fmt::Display for ModulePath<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate")?;
        self.segments().iter().try_for_each(|segment| write!(formatter, "::{segment}"))
    }
}

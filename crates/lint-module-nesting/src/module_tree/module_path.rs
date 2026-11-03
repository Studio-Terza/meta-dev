mod ancestor;
mod index;
mod inside_sibling;
mod relation;

pub use ancestor::Ancestor;
pub use index::ModuleIndex;
pub use inside_sibling::InsideSibling;
pub use relation::Relation;

use std::fmt;
use std::path::Path;

use lint_core::SourceFile;
use lint_layout::UsePath;
use lint_layout::{Name, Whole, Word, words};
use lint_placement::CrateRoot;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModulePath<'file> {
    crate_root: CrateRoot,
    segments: Vec<Name>,
    file: &'file Path,
}

impl<'file> ModulePath<'file> {
    const CRATE: &'static str = "crate";
    const SUPER: &'static str = "super";
    const ANCHORS: [Word<'static>; 3] = words!["crate", "super", "self"];

    pub fn of_file(file: &'file SourceFile) -> ModulePath<'file> {
        ModulePath {
            crate_root: file.placement().crate_root().clone(),
            segments: file.placement().module().segments().to_vec(),
            file: file.path(),
        }
    }

    pub fn from_use(
        file: &ModulePath<'file>,
        path: &UsePath,
        modules: &ModuleIndex<'file>,
    ) -> Option<ModulePath<'file>> {
        let first = path.segments().first()?;
        let start = if *first == Self::CRATE {
            Vec::new()
        } else if *first == Self::SUPER {
            file.parent()?.segments
        } else {
            file.segments.clone()
        };
        let anchor = first.matches::<Whole>(&Self::ANCHORS);
        let child = file.child(first);
        anchor
            .map(|_| ())
            .or_else(|| modules.deepest(&child).filter(|found| found.segments == child.segments).map(|_| ()))?;
        let skip = anchor.into_iter().count();
        let segments = path.segments().iter().skip(skip).try_fold(start, |mut segments, segment| {
            if *segment == Self::SUPER {
                segments.pop().and(Some(segments))
            } else {
                segments.push(segment.clone());
                Some(segments)
            }
        })?;
        Some(ModulePath { crate_root: file.crate_root.clone(), segments, file: Path::new("") })
    }

    pub fn file(&self) -> &'file Path {
        self.file
    }

    pub fn parent(&self) -> Option<ModulePath<'file>> {
        let (_, parents) = self.segments.split_last()?;
        Some(ModulePath { crate_root: self.crate_root.clone(), segments: parents.to_vec(), file: Path::new("") })
    }

    pub fn truncated_like(&self, other: &ModulePath<'file>) -> ModulePath<'file> {
        ModulePath {
            crate_root: self.crate_root.clone(),
            segments: self.segments.iter().take(other.segments.len()).cloned().collect(),
            file: Path::new(""),
        }
    }

    fn child(&self, name: &Name) -> ModulePath<'file> {
        ModulePath {
            crate_root: self.crate_root.clone(),
            segments: self.segments.iter().cloned().chain(Some(name.clone())).collect(),
            file: Path::new(""),
        }
    }
}

impl fmt::Display for ModulePath<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate")?;
        self.segments.iter().try_for_each(|segment| write!(formatter, "::{segment}"))
    }
}

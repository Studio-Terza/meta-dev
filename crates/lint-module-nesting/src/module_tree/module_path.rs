mod ancestor;
mod index;
mod inside_sibling;
mod relation;

pub use ancestor::Ancestor;
pub use index::ModuleIndex;
pub use inside_sibling::InsideSibling;
pub use relation::Relation;

use std::fmt;
use std::path::{Path, PathBuf};

use lint_core::SourceFile;
use lint_layout::UsePath;
use lint_layout::{Name, Whole, Word, words};
use lint_placement::CrateRoot;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModulePath {
    crate_root: CrateRoot,
    segments: Vec<Name>,
    file: PathBuf,
}

impl ModulePath {
    const MODULE_FILE: &'static str = "mod.rs";
    const CRATE: &'static str = "crate";
    const SUPER: &'static str = "super";
    const ANCHORS: [Word<'static>; 3] = words!["crate", "super", "self"];

    pub fn of_file(file: &SourceFile) -> ModulePath {
        ModulePath {
            crate_root: file.placement().crate_root().clone(),
            segments: file.placement().module().segments().to_vec(),
            file: file.path().to_path_buf(),
        }
    }

    pub fn from_use(file: &ModulePath, path: &UsePath, modules: &ModuleIndex) -> Option<ModulePath> {
        let first = path.first()?;
        let start = if *first == Self::CRATE {
            Vec::new()
        } else if *first == Self::SUPER {
            file.parent()?.segments
        } else {
            file.segments.clone()
        };
        let anchor = first.matches::<Whole>(&Self::ANCHORS);
        anchor.map(|_| ()).or_else(|| modules.same_as(&file.child(first)).map(|_| ()))?;
        let skip = anchor.into_iter().count();
        let segments = path.segments().iter().skip(skip).try_fold(start, |mut segments, segment| {
            if *segment == Self::SUPER {
                segments.pop().and(Some(segments))
            } else {
                segments.push(segment.clone());
                Some(segments)
            }
        })?;
        Some(ModulePath { crate_root: file.crate_root.clone(), segments, file: PathBuf::new() })
    }

    pub fn file(&self) -> &Path {
        &self.file
    }

    pub fn parent(&self) -> Option<ModulePath> {
        let (_, parents) = self.segments.split_last()?;
        Some(ModulePath { crate_root: self.crate_root.clone(), segments: parents.to_vec(), file: PathBuf::new() })
    }

    pub fn relates<'other, R: Relation>(&self, other: &'other ModulePath) -> Option<&'other ModulePath> {
        R::holds(self, other)
    }

    pub fn crossed_directory(&self, user: &ModulePath, modules: &ModuleIndex) -> Option<ModulePath> {
        self.prefixes().take(self.segments.len()).skip(1).find(|boundary| {
            modules.directory_same_as(boundary).is_some() && boundary.relates::<Ancestor>(user).is_none()
        })
    }

    pub fn deepest_in<'index>(&self, modules: &'index ModuleIndex) -> Option<&'index ModulePath> {
        self.prefixes().rev().find_map(|probe| modules.same_as(&probe))
    }

    pub fn truncated_like(&self, other: &ModulePath) -> ModulePath {
        self.with_segments(self.segments.iter().take(other.segments.len()).cloned().collect())
    }

    fn prefixes(&self) -> impl DoubleEndedIterator<Item = ModulePath> + '_ {
        (0..=self.segments.len()).map(|length| self.with_segments(self.segments.iter().take(length).cloned().collect()))
    }

    fn with_segments(&self, segments: Vec<Name>) -> ModulePath {
        ModulePath { crate_root: self.crate_root.clone(), segments, file: PathBuf::new() }
    }

    fn child(&self, name: &Name) -> ModulePath {
        ModulePath {
            crate_root: self.crate_root.clone(),
            segments: self.segments.iter().cloned().chain(Some(name.clone())).collect(),
            file: PathBuf::new(),
        }
    }

    fn directory_module(&self) -> Option<&ModulePath> {
        self.file.file_name().filter(|name| *name == Self::MODULE_FILE).map(|_| self)
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate")?;
        self.segments.iter().try_for_each(|segment| write!(formatter, "::{segment}"))
    }
}

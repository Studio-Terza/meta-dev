mod ancestor;
mod inside_sibling;
mod relation;
mod same;

pub use ancestor::Ancestor;
pub use inside_sibling::InsideSibling;
pub use relation::Relation;
pub use same::Same;

use std::fmt;
use std::path::{Path, PathBuf};

use lint_core::SourceFile;
use lint_layout::UsePath;
use lint_layout::{Name, Whole};
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
    const ANCHORS: [&'static str; 3] = ["crate", "super", "self"];

    pub fn of_file(file: &SourceFile) -> ModulePath {
        ModulePath {
            crate_root: file.placement().crate_root().clone(),
            segments: file.placement().module().segments().to_vec(),
            file: file.path().to_path_buf(),
        }
    }

    pub fn from_use(file: &ModulePath, path: &UsePath, modules: &[ModulePath]) -> Option<ModulePath> {
        let first = path.first()?;
        let start = if *first == Self::CRATE {
            Vec::new()
        } else if *first == Self::SUPER {
            file.parent()?.segments
        } else {
            file.segments.clone()
        };
        let skip = usize::from(first.matches::<Whole>(&Self::ANCHORS));
        (skip != 0 || modules.iter().any(|module| module.relates::<Same>(&file.child(first)))).then_some(())?;
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

    pub fn deepest_module_containing<'tree>(
        modules: &'tree [ModulePath],
        target: &ModulePath,
        user: &ModulePath,
    ) -> Option<&'tree ModulePath> {
        modules
            .iter()
            .filter(|module| module.relates::<Ancestor>(target) && !module.relates::<Same>(user))
            .max_by_key(|module| module.segments.len())
    }

    pub fn file(&self) -> &Path {
        &self.file
    }

    pub fn segment_length(&self) -> usize {
        self.segments.len()
    }

    pub fn parent(&self) -> Option<ModulePath> {
        let (_, parents) = self.segments.split_last()?;
        Some(ModulePath { crate_root: self.crate_root.clone(), segments: parents.to_vec(), file: PathBuf::new() })
    }

    pub fn relates<R: Relation>(&self, other: &ModulePath) -> bool {
        R::holds(self, other)
    }

    pub fn crosses_directory_boundary(&self, user: &ModulePath, modules: &[ModulePath]) -> bool {
        (1..self.segments.len()).any(|cut| {
            let boundary = self.truncated_to_length(cut);
            modules.iter().any(|module| module.relates::<Same>(&boundary) && module.is_directory_module())
                && !boundary.relates::<Ancestor>(user)
        })
    }

    pub fn truncated_to_length(&self, length: usize) -> ModulePath {
        ModulePath {
            crate_root: self.crate_root.clone(),
            segments: self.segments.iter().take(length).cloned().collect(),
            file: PathBuf::new(),
        }
    }

    fn child(&self, name: &Name) -> ModulePath {
        ModulePath {
            crate_root: self.crate_root.clone(),
            segments: self.segments.iter().cloned().chain(Some(name.clone())).collect(),
            file: PathBuf::new(),
        }
    }

    fn is_directory_module(&self) -> bool {
        self.file.file_name().is_some_and(|name| name == Self::MODULE_FILE)
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate")?;
        self.segments.iter().try_for_each(|segment| write!(formatter, "::{segment}"))
    }
}

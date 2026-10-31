mod ancestor;
mod inside_sibling;
mod relation;
mod same;

pub use ancestor::Ancestor;
pub use inside_sibling::InsideSibling;
pub use relation::Relation;
pub use same::Same;

use std::fmt;
use std::path::{Component, Path, PathBuf};

use lint_core::UsePath;
use lint_core::{Name, Whole};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModulePath {
    crate_root: PathBuf,
    segments: Vec<Name>,
    file: PathBuf,
}

impl ModulePath {
    const SOURCE_ROOTS: [&'static str; 2] = ["src/lib.rs", "src/main.rs"];
    const SOURCES: &'static str = "src";
    const MODULE_FILE: &'static str = "mod.rs";
    const CRATE: &'static str = "crate";
    const SUPER: &'static str = "super";
    const ANCHORS: [&'static str; 3] = ["crate", "super", "self"];

    pub fn of_file(path: &Path) -> Option<ModulePath> {
        let components: Vec<Component> = path.components().collect();
        let source = components.iter().position(|component| component.as_os_str() == Self::SOURCES)?;
        let crate_root: PathBuf = components.iter().take(source).collect();
        let inside: PathBuf = components.iter().skip(source + 1).collect();
        let stem = inside.file_stem()?.to_string_lossy().into_owned();
        let segments = inside
            .parent()
            .into_iter()
            .flat_map(Path::components)
            .map(|component| Name::from(&*component.as_os_str().to_string_lossy()))
            .chain(
                (!Self::is_crate_root(path) && inside.file_name().is_some_and(|name| name != Self::MODULE_FILE))
                    .then(|| Name::from(&*stem)),
            )
            .collect();
        Some(ModulePath { crate_root, segments, file: path.to_path_buf() })
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

    pub fn segments(&self) -> &[Name] {
        &self.segments
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

    fn is_crate_root(file: &Path) -> bool {
        Self::SOURCE_ROOTS.iter().any(|root| file.ends_with(root))
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "crate")?;
        self.segments.iter().try_for_each(|segment| write!(formatter, "::{segment}"))
    }
}

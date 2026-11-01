mod build_script;
mod source_directory;
mod target_directory;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::crate_file::CrateFile;
use crate::crate_root::CrateRoot;
use crate::module_path::ModulePath;
use crate::package_root::PackageRoot;
use crate::role::Role;
use build_script::BuildScript;
use source_directory::SourceDirectory;
use target_directory::TargetDirectory;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    package: PackageRoot,
    crate_root: CrateRoot,
    module: ModulePath,
    role: Role,
}

impl Placement {
    const TARGET_DIRECTORIES: [&'static str; 3] = ["tests", "examples", "benches"];
    const SOURCE_DIRECTORY: &'static str = "src";
    const LIBRARY_FILE: &'static str = "lib.rs";
    const BUILD_FILE: &'static str = "build.rs";

    #[must_use]
    pub fn locate(path: &Path) -> Placement {
        Self::within(&Self::guessed_package(path), path)
    }

    #[must_use]
    pub fn within(package: &Path, file: &Path) -> Placement {
        file.strip_prefix(package)
            .ok()
            .and_then(|inside| Self::in_package(package, inside))
            .unwrap_or_else(|| Self::unplaced(CrateRoot::new(file.to_path_buf())))
    }

    #[must_use]
    pub fn package(&self) -> &PackageRoot {
        &self.package
    }

    #[must_use]
    pub fn crate_root(&self) -> &CrateRoot {
        &self.crate_root
    }

    #[must_use]
    pub fn module(&self) -> &ModulePath {
        &self.module
    }

    #[must_use]
    pub fn role(&self) -> Role {
        self.role
    }

    #[must_use]
    pub fn crate_file(&self, file: &Path) -> CrateFile {
        let source = self.crate_root.directory().join(Self::SOURCE_DIRECTORY);
        CrateFile::from(match self.role {
            Role::Other if file.starts_with(&source) => source.join(Self::LIBRARY_FILE),
            Role::Other | Role::LibraryRoot | Role::BinaryRoot | Role::Target => file.to_path_buf(),
        })
    }

    fn named_ancestor<'path>(path: &'path Path, accepted: &[&str]) -> Option<&'path Path> {
        path.ancestors()
            .skip(1)
            .find(|ancestor| ancestor.file_name().and_then(OsStr::to_str).is_some_and(|name| accepted.contains(&name)))
    }

    fn guessed_package(path: &Path) -> PathBuf {
        Self::named_ancestor(path, &[Self::SOURCE_DIRECTORY])
            .or_else(|| Self::named_ancestor(path, &Self::TARGET_DIRECTORIES))
            .and_then(Path::parent)
            .or_else(|| path.parent())
            .map(Path::to_path_buf)
            .unwrap_or_default()
    }

    fn unplaced(crate_root: CrateRoot) -> Placement {
        Placement {
            package: PackageRoot::unplaced(crate_root.clone()),
            crate_root,
            module: ModulePath::default(),
            role: Role::Other,
        }
    }

    fn in_package(package: &Path, inside: &Path) -> Option<Placement> {
        let first = inside.components().next()?;
        let rest = inside.strip_prefix(first).ok()?;
        let directory = first.as_os_str().to_str()?;
        let root = PackageRoot::read(package);
        if directory == Self::SOURCE_DIRECTORY {
            Some(SourceDirectory::place(root, rest))
        } else if directory == Self::BUILD_FILE && rest.as_os_str().is_empty() {
            Some(BuildScript::place(root))
        } else {
            Self::TARGET_DIRECTORIES
                .contains(&directory)
                .then(|| TargetDirectory::place(root, &package.join(directory), rest))
                .flatten()
        }
    }
}

mod build_script;
mod source_directory;
mod target_directory;

use std::path::{Path, PathBuf};

use crate::crate_file::CrateFile;
use crate::crate_root::CrateRoot;
use crate::module_path::ModulePath;
use crate::package_root::PackageRoot;
use crate::packages::Packages;
use crate::role::Role;
use build_script::BuildScript;
use file_ports::Files;
use lint_layout::{Name, Whole, Word};
use source_directory::SourceDirectory;
use target_directory::TargetDirectory;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    package: PackageRoot,
    target_root: Option<CrateRoot>,
    module: ModulePath,
    role: Role,
}

impl Placement {
    const TARGET_DIRECTORIES: [&'static str; 3] = ["tests", "examples", "benches"];
    const SOURCE_DIRECTORY: &'static str = "src";
    const LIBRARY_FILE: &'static str = "lib.rs";
    const BUILD_FILE: &'static str = "build.rs";

    #[must_use]
    pub fn locate(files: &impl Files, path: &Path) -> Placement {
        Self::within(files, &Self::guessed_package(path), path)
    }

    #[must_use]
    pub fn within(files: &impl Files, package: &Path, file: &Path) -> Placement {
        Packages::read(files, &[package]).placed(package, file)
    }

    pub(crate) fn placed(root: Option<&PackageRoot>, package: &Path, file: &Path) -> Placement {
        file.strip_prefix(package)
            .ok()
            .zip(root.cloned())
            .and_then(|(inside, package_root)| Self::in_package(package_root, package, inside))
            .unwrap_or_else(|| Self::unplaced(CrateRoot::new(file.to_path_buf())))
    }

    #[must_use]
    pub fn package(&self) -> &PackageRoot {
        &self.package
    }

    #[must_use]
    pub fn crate_root(&self) -> &CrateRoot {
        self.target_root.as_ref().unwrap_or_else(|| self.package.root())
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
    pub fn executable_root<'file>(&self, file: &'file Path) -> Option<&'file Path> {
        Some(file).filter(|candidate| {
            self.role.roots(self.crate_root().directory(), candidate).iter().any(|root| root == candidate)
        })
    }

    #[must_use]
    pub fn crate_file(&self, file: &Path) -> CrateFile {
        let source = self.crate_root().directory().join(Self::SOURCE_DIRECTORY);
        let library =
            (self.role == Role::Other).then(|| source.join(Self::LIBRARY_FILE)).filter(|_| file.starts_with(&source));
        CrateFile::from(library.unwrap_or_else(|| file.to_path_buf()))
    }

    fn named_ancestor<'path>(path: &'path Path, accepted: &[Word<'_>]) -> Option<&'path Path> {
        path.ancestors().skip(1).find(|ancestor| {
            ancestor.file_name().map(Name::from).and_then(|name| name.matches::<Whole>(accepted)).is_some()
        })
    }

    fn guessed_package(path: &Path) -> PathBuf {
        Self::named_ancestor(path, &[Word::new(Self::SOURCE_DIRECTORY)])
            .or_else(|| Self::named_ancestor(path, &Self::TARGET_DIRECTORIES.map(Word::new)))
            .and_then(Path::parent)
            .or_else(|| path.parent())
            .map(Path::to_path_buf)
            .unwrap_or_default()
    }

    fn unplaced(crate_root: CrateRoot) -> Placement {
        Placement {
            package: PackageRoot::unplaced(crate_root),
            target_root: None,
            module: ModulePath::default(),
            role: Role::Other,
        }
    }

    fn in_package(root: PackageRoot, package: &Path, inside: &Path) -> Option<Placement> {
        let first = inside.components().next()?;
        let rest = inside.strip_prefix(first).ok()?;
        let directory = first.as_os_str().to_str()?;
        PackageDirectory::from(directory).placed(root, &package.join(directory), rest)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PackageDirectory {
    Source,
    Build,
    Target,
    Other,
}

impl PackageDirectory {
    const NAMED: [(&'static str, PackageDirectory); 2] =
        [(Placement::SOURCE_DIRECTORY, PackageDirectory::Source), (Placement::BUILD_FILE, PackageDirectory::Build)];
}

impl From<&str> for PackageDirectory {
    fn from(directory: &str) -> PackageDirectory {
        PackageDirectory::NAMED
            .iter()
            .find(|(name, _)| *name == directory)
            .map(|(_, named)| *named)
            .or_else(|| {
                Placement::TARGET_DIRECTORIES
                    .iter()
                    .find(|target| **target == directory)
                    .map(|_| PackageDirectory::Target)
            })
            .unwrap_or(PackageDirectory::Other)
    }
}

impl PackageDirectory {
    fn placed(self, root: PackageRoot, directory: &Path, rest: &Path) -> Option<Placement> {
        match self {
            PackageDirectory::Source => Some(SourceDirectory::place(root, rest)),
            PackageDirectory::Build => {
                Some(rest).filter(|candidate| candidate.as_os_str().is_empty()).map(|_| BuildScript::place(root))
            }
            PackageDirectory::Target => TargetDirectory::place(root, directory, rest),
            PackageDirectory::Other => None,
        }
    }
}

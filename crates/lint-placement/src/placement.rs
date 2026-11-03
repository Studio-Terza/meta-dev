mod build_script;
mod source_directory;
mod target_directory;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::crate_file::CrateFile;
use crate::crate_root::CrateRoot;
use crate::module_path::ModulePath;
use crate::package_root::PackageRoot;
use crate::role::Role;
use build_script::BuildScript;
use file_ports::Files;
use lint_layout::{Name, Whole, Word};
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
    pub fn locate(files: &impl Files, path: &Path) -> Placement {
        Self::within(files, &Self::guessed_package(path), path)
    }

    #[must_use]
    pub fn within(files: &impl Files, package: &Path, file: &Path) -> Placement {
        Self::within_many(files, &[package.to_path_buf()], &[file.to_path_buf()])
            .pop()
            .unwrap_or_else(|| Self::unplaced(CrateRoot::new(file.to_path_buf())))
    }

    #[must_use]
    pub fn within_many(files: &impl Files, packages: &[PathBuf], paths: &[PathBuf]) -> Vec<Placement> {
        let distinct: Vec<PathBuf> = packages.iter().cloned().collect::<BTreeSet<PathBuf>>().into_iter().collect();
        let roots: BTreeMap<PathBuf, PackageRoot> =
            distinct.iter().cloned().zip(PackageRoot::read_many(files, &distinct)).collect();
        packages
            .iter()
            .zip(paths)
            .map(|(package, file)| {
                file.strip_prefix(package)
                    .ok()
                    .zip(roots.get(package).cloned())
                    .and_then(|(inside, root)| Self::in_package(root, package, inside))
                    .unwrap_or_else(|| Self::unplaced(CrateRoot::new(file.clone())))
            })
            .collect()
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
            package: PackageRoot::unplaced(crate_root.clone()),
            crate_root,
            module: ModulePath::default(),
            role: Role::Other,
        }
    }

    fn in_package(root: PackageRoot, package: &Path, inside: &Path) -> Option<Placement> {
        let first = inside.components().next()?;
        let rest = inside.strip_prefix(first).ok()?;
        let directory = first.as_os_str().to_str()?;
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

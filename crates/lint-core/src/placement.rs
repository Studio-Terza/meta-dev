use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::module_path::ModulePath;
use crate::role::Role;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    crate_root: PathBuf,
    module: ModulePath,
    role: Role,
}

impl Placement {
    const TARGET_DIRECTORIES: [&'static str; 3] = ["tests", "examples", "benches"];

    #[must_use]
    pub fn of(path: &Path) -> Placement {
        Self::in_source_directory(path).or_else(|| Self::in_target_directory(path)).unwrap_or_else(|| Placement {
            crate_root: path.to_path_buf(),
            module: ModulePath::default(),
            role: Role::Other,
        })
    }

    #[must_use]
    pub fn crate_root(&self) -> &Path {
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

    fn named_ancestor<'path>(path: &'path Path, accepted: &[&str]) -> Option<&'path Path> {
        path.ancestors().skip(1).find(|ancestor| {
            ancestor.file_name().is_some_and(|name| accepted.iter().any(|candidate| name == OsStr::new(candidate)))
        })
    }

    fn in_source_directory(path: &Path) -> Option<Placement> {
        let source = Self::named_ancestor(path, &["src"])?;
        let inside = path.strip_prefix(source).ok()?;
        let role = Self::binary_role(inside);
        let module = match role {
            Role::BinaryRoot => ModulePath::default(),
            Role::Other if inside == Path::new("lib.rs") => ModulePath::default(),
            Role::Other => ModulePath::of_file(inside),
        };
        Some(Placement { crate_root: source.parent().unwrap_or(Path::new("")).to_path_buf(), module, role })
    }

    fn binary_role(inside: &Path) -> Role {
        let direct = inside == Path::new("main.rs");
        let in_bin = inside.parent() == Some(Path::new("bin"));
        let bin_directory = inside.file_name() == Some(OsStr::new("main.rs"))
            && inside.parent().and_then(Path::parent) == Some(Path::new("bin"));
        if direct || in_bin || bin_directory { Role::BinaryRoot } else { Role::Other }
    }

    fn in_target_directory(path: &Path) -> Option<Placement> {
        let directory = Self::named_ancestor(path, &Self::TARGET_DIRECTORIES)?;
        let inside = path.strip_prefix(directory).ok()?;
        let first = inside.components().next()?;
        let rest = inside.strip_prefix(first).ok()?;
        let module = if rest.as_os_str().is_empty() { ModulePath::default() } else { ModulePath::of_file(rest) };
        Some(Placement { crate_root: directory.join(first), module, role: Role::Other })
    }
}

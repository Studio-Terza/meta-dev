use std::ffi::OsStr;
use std::path::Path;

use crate::crate_root::CrateRoot;
use crate::module_path::ModulePath;
use crate::role::Role;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    crate_root: CrateRoot,
    module: ModulePath,
    role: Role,
}

impl Placement {
    const TARGET_DIRECTORIES: [&'static str; 3] = ["tests", "examples", "benches"];
    const SOURCE_DIRECTORIES: [&'static str; 1] = ["src"];
    const LIBRARY_ROOT: &'static str = "lib.rs";

    #[must_use]
    pub fn locate(path: &Path) -> Placement {
        Self::in_source_directory(path)
            .or_else(|| Self::in_target_directory(path, &Self::TARGET_DIRECTORIES))
            .unwrap_or_else(|| Placement {
                crate_root: CrateRoot::new(path.to_path_buf()),
                module: ModulePath::default(),
                role: Role::Other,
            })
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

    fn named_ancestor<'path>(path: &'path Path, accepted: &[&str]) -> Option<&'path Path> {
        path.ancestors()
            .skip(1)
            .find(|ancestor| ancestor.file_name().and_then(OsStr::to_str).is_some_and(|name| accepted.contains(&name)))
    }

    fn in_source_directory(path: &Path) -> Option<Placement> {
        let source = Self::named_ancestor(path, &Self::SOURCE_DIRECTORIES)?;
        let inside = path.strip_prefix(source).ok()?;
        let role = Role::in_source(inside);
        let module = match role {
            Role::BinaryRoot => ModulePath::default(),
            Role::Other | Role::Target if inside == Path::new(Self::LIBRARY_ROOT) => ModulePath::default(),
            Role::Other | Role::Target => ModulePath::of_file(inside),
        };
        Some(Placement {
            crate_root: CrateRoot::new(source.parent().unwrap_or(Path::new("")).to_path_buf()),
            module,
            role,
        })
    }

    fn in_target_directory(path: &Path, directories: &[&str]) -> Option<Placement> {
        let directory = Self::named_ancestor(path, directories)?;
        let inside = path.strip_prefix(directory).ok()?;
        let first = inside.components().next()?;
        let rest = inside.strip_prefix(first).ok()?;
        let module = if rest.as_os_str().is_empty() { ModulePath::default() } else { ModulePath::of_file(rest) };
        Some(Placement { crate_root: CrateRoot::new(directory.join(first)), module, role: Role::Target })
    }
}

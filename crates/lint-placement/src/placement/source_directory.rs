use std::path::Path;

use super::Placement;
use crate::module_path::ModulePath;
use crate::package_root::PackageRoot;
use crate::role::Role;

#[derive(Debug)]
pub struct SourceDirectory;

impl SourceDirectory {
    const MAIN_FILE: &'static str = "main.rs";

    #[must_use]
    pub fn place(package: PackageRoot, inside: &Path) -> Placement {
        let role = Role::in_source(inside);
        let module = Self::module(role, inside);
        Placement { target_root: None, package, module, role }
    }

    fn module(role: Role, inside: &Path) -> ModulePath {
        match role {
            Role::LibraryRoot => ModulePath::default(),
            Role::BinaryRoot => Some(inside)
                .filter(|inside| *inside == Path::new(Self::MAIN_FILE))
                .map_or_else(|| ModulePath::of_file(inside), |_| ModulePath::default()),
            Role::Other | Role::Target => ModulePath::of_file(inside),
        }
    }
}

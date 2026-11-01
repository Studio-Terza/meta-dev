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
        let module = match role {
            Role::LibraryRoot => ModulePath::default(),
            Role::BinaryRoot if inside == Path::new(Self::MAIN_FILE) => ModulePath::default(),
            Role::BinaryRoot | Role::Other | Role::Target => ModulePath::of_file(inside),
        };
        Placement { crate_root: package.root().clone(), package, module, role }
    }
}

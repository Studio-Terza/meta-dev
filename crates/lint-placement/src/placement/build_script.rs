use super::Placement;
use crate::module_path::ModulePath;
use crate::package_root::PackageRoot;
use crate::role::Role;

#[derive(Debug)]
pub struct BuildScript;

impl BuildScript {
    #[must_use]
    pub fn place(package: PackageRoot) -> Placement {
        Placement { crate_root: package.root().clone(), package, module: ModulePath::default(), role: Role::Other }
    }
}

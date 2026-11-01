use std::path::Path;

use super::Placement;
use crate::crate_root::CrateRoot;
use crate::module_path::ModulePath;
use crate::role::Role;

#[derive(Debug)]
pub struct TargetDirectory;

impl TargetDirectory {
    #[must_use]
    pub fn place(directory: &Path, inside: &Path) -> Option<Placement> {
        let first = inside.components().next()?;
        let rest = inside.strip_prefix(first).ok()?;
        let module = if rest.as_os_str().is_empty() { ModulePath::default() } else { ModulePath::of_file(rest) };
        Some(Placement { crate_root: CrateRoot::new(directory.join(first)), module, role: Role::Target })
    }
}
